//! Ed25519 signing and verification for managed policy files.
//!
//! Policies can be cryptographically signed to guarantee authenticity.
//! Key pairs are stored in the Onus config directory:
//!   - `{config_dir}/signing/signing_key.pem`  (private key, secret)
//!   - `{config_dir}/signing/signing_pub.pem`   (public key)
//!
//! A signed policy contains a `[signature]` section with:
//!   - `algorithm = "ed25519"`
//!   - `signer = "<public-key-hex>"` (identifies who signed it)
//!   - `value = "<base64-signature>"` (the actual signature)

use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use std::path::PathBuf;

const BASE64_ENGINE: base64::engine::GeneralPurpose =
    base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, base64::engine::GeneralPurposeConfig::new());

/// Information about a policy signature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignatureInfo {
    pub algorithm: String,
    pub signer: String,
    pub value: String,
}

impl SignatureInfo {
    pub fn is_empty(&self) -> bool {
        self.algorithm.is_empty() || self.value.is_empty()
    }
}

/// Wrapper for a policy that may be signed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignedPolicy {
    #[serde(flatten)]
    pub body: serde_json::Value,
    #[serde(default)]
    pub signature: Option<SignatureInfo>,
}

impl SignedPolicy {
    /// Returns the canonical bytes to sign/verify (the body without the signature field).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut body = self.body.clone();
        if let Some(obj) = body.as_object_mut() {
            obj.remove("signature");
        }
        serde_json::to_vec(&body).unwrap_or_default()
    }

    /// Verify the signature against the body.
    pub fn verify(&self) -> Result<(), String> {
        let sig = self
            .signature
            .as_ref()
            .ok_or_else(|| "No signature present".to_string())?;

        if sig.algorithm != "ed25519" {
            return Err(format!("Unsupported algorithm: {}", sig.algorithm));
        }

        let pub_key_bytes =
            hex::decode(&sig.signer).map_err(|e| format!("Invalid public key hex: {}", e))?;

        if pub_key_bytes.len() != 32 {
            return Err("Public key must be 32 bytes (64 hex chars)".to_string());
        }

        let sig_bytes =
            base64::Engine::decode(&BASE64_ENGINE, &sig.value).map_err(|e| format!("Invalid signature base64: {}", e))?;

        let body_bytes = self.canonical_bytes();
        let public_key = UnparsedPublicKey::new(&ED25519, pub_key_bytes);
        public_key
            .verify(&body_bytes, &sig_bytes)
            .map_err(|_| "Signature verification FAILED".to_string())
    }
}

// ── Key management ───────────────────────────────────────────────────────────

/// Path to the signing directory.
pub fn signing_dir() -> PathBuf {
    crate::config_dir().join("signing")
}

/// Path to the private signing key.
pub fn private_key_path() -> PathBuf {
    signing_dir().join("signing_key.pem")
}

/// Path to the public signing key.
pub fn public_key_path() -> PathBuf {
    signing_dir().join("signing_pub.pem")
}

/// Check whether signing keys exist.
pub fn has_signing_keys() -> bool {
    private_key_path().exists() && public_key_path().exists()
}

/// Generate a new Ed25519 key pair and save to disk.
pub fn generate_keys() -> anyhow::Result<()> {
    let dir = signing_dir();
    std::fs::create_dir_all(&dir)?;

    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
        .map_err(|e| anyhow::anyhow!("Failed to generate Ed25519 key: {}", e))?;

    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .map_err(|e| anyhow::anyhow!("Failed to parse generated key: {}", e))?;

    let private_pem = pem::encode(&pem::Pem::new("PRIVATE KEY", pkcs8_bytes.as_ref().to_vec()));
    let pub_key_bytes = key_pair.public_key().as_ref();
    let public_hex = hex::encode(pub_key_bytes);
    let public_pem = pem::encode(&pem::Pem::new("PUBLIC KEY", pub_key_bytes.to_vec()));

    std::fs::write(private_key_path(), private_pem.as_bytes())?;
    std::fs::write(public_key_path(), format!("{}\n# Public key (hex): {}\n", public_pem, public_hex).as_bytes())?;

    log::info!("Signing keys generated at {}", dir.display());
    Ok(())
}

/// Load the private signing key.
pub fn load_private_key() -> anyhow::Result<Ed25519KeyPair> {
    let pem_data = std::fs::read_to_string(private_key_path())?;
    let pem_parsed = pem::parse(&pem_data)
        .map_err(|e| anyhow::anyhow!("Failed to parse private key PEM: {}", e))?;
    let key_pair = Ed25519KeyPair::from_pkcs8(pem_parsed.contents())
        .map_err(|e| anyhow::anyhow!("Failed to load private key: {}", e))?;
    Ok(key_pair)
}

/// Load the public key hex string.
pub fn load_public_key_hex() -> anyhow::Result<String> {
    let pem_data = std::fs::read_to_string(public_key_path())?;
    // Extract hex public key from comment line: "# Public key (hex): <hex>"
    for line in pem_data.lines() {
        if let Some(hex_str) = line.strip_prefix("# Public key (hex): ") {
            return Ok(hex_str.trim().to_string());
        }
    }
    // Fallback: parse PEM directly to get key bytes
    let pem_parsed = pem::parse(&pem_data)
        .map_err(|e| anyhow::anyhow!("Failed to parse public key PEM: {}", e))?;
    Ok(hex::encode(pem_parsed.contents()))
}

/// Sign a policy's canonical bytes and return SignatureInfo.
/// Uses canonical_bytes() so the sign and verify paths are consistent.
pub fn sign_policy(policy: &SignedPolicy, key_pair: &Ed25519KeyPair) -> anyhow::Result<SignatureInfo> {
    let canonical = policy.canonical_bytes();
    let signature = key_pair.sign(&canonical);
    let pub_key_hex = hex::encode(key_pair.public_key().as_ref());
    Ok(SignatureInfo {
        algorithm: "ed25519".to_string(),
        signer: pub_key_hex,
        value: base64::Engine::encode(&BASE64_ENGINE, signature.as_ref()),
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_sign_roundtrip() {
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let pub_hex = hex::encode(key_pair.public_key().as_ref());

        let mut signed_policy = SignedPolicy {
            body: serde_json::json!({"name": "test-rule", "action": "block"}),
            signature: None,
        };
        let sig = sign_policy(&signed_policy, &key_pair).unwrap();
        signed_policy.signature = Some(sig);

        assert_eq!(signed_policy.signature.as_ref().unwrap().algorithm, "ed25519");
        assert_eq!(signed_policy.signature.as_ref().unwrap().signer, pub_hex);

        // Verify roundtrip
        assert!(signed_policy.verify().is_ok());
    }

    #[test]
    fn test_verify_fails_on_tampered_body() {
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();

        let original = SignedPolicy {
            body: serde_json::json!({"name": "test-rule", "action": "block"}),
            signature: None,
        };
        let sig = sign_policy(&original, &key_pair).unwrap();
        let tampered = SignedPolicy {
            body: serde_json::json!({"name": "evil-rule", "action": "allow"}),
            signature: Some(sig),
        };
        assert!(tampered.verify().is_err());
    }

    #[test]
    fn test_verify_fails_without_signature() {
        let signed_policy = SignedPolicy {
            body: serde_json::json!({"name": "test"}),
            signature: None,
        };
        assert!(signed_policy.verify().is_err());
    }

    #[test]
    fn test_verify_fails_on_tampered_signature() {
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();

        let mut signed_policy = SignedPolicy {
            body: serde_json::json!({"name": "test-rule", "action": "block"}),
            signature: None,
        };
        let sig = sign_policy(&signed_policy, &key_pair).unwrap();
        let mut tampered_sig = sig;
        tampered_sig.value = "AAAA".to_string(); // Corrupt the signature
        signed_policy.signature = Some(tampered_sig);

        assert!(signed_policy.verify().is_err());
    }
}
