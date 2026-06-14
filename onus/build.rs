//! Build script — embeds rules/default.toml into the binary
//! so `onus rules init` works even without access to GitHub.

fn main() {
    // Re-run build.rs if the rules file changes.
    println!("cargo:rerun-if-changed=rules/default.toml");

    // Tell Cargo to embed the file.
    // The include_str! macro in rules.rs already handles this at compile time.
}
