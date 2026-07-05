use std::fs;
use std::io;
use std::path::Path;

const ENV_FILE_NAME: &str = "onus.env";

pub fn load_default_env_file() -> io::Result<()> {
    let path = crate::config_dir().join(ENV_FILE_NAME);
    load_env_file_if_present(&path)
}

pub fn load_env_file_if_present(path: &Path) -> io::Result<()> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            load_env_contents(&contents);
            Ok(())
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

pub fn load_env_contents(contents: &str) {
    for line in contents.lines() {
        let Some((key, value)) = parse_env_line(line) else {
            continue;
        };
        if env_key_allowed(&key) && std::env::var_os(&key).is_none() {
            std::env::set_var(key, value);
        }
    }
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let assignment = trimmed.strip_prefix("export ").unwrap_or(trimmed);
    let (raw_key, raw_value) = assignment.split_once('=')?;
    let key = raw_key.trim();
    if !valid_key(key) {
        return None;
    }
    Some((key.to_string(), strip_quotes(raw_value.trim()).to_string()))
}

fn strip_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn env_key_allowed(key: &str) -> bool {
    key == "RUST_LOG" || key.starts_with("ONUS_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_only_onus_and_logging_keys_without_overwriting_existing_env() {
        std::env::set_var("ONUS_EXISTING_TEST_VALUE", "keep");
        std::env::remove_var("ONUS_NEW_TEST_VALUE");
        std::env::remove_var("RUST_LOG_TEST_SHOULD_NOT_LOAD");
        load_env_contents(
            r#"
            # ignored
            ONUS_EXISTING_TEST_VALUE=replace
            ONUS_NEW_TEST_VALUE="loaded"
            RUST_LOG=debug
            PATH=/tmp/not-loaded
            invalid-key=value
            "#,
        );
        assert_eq!(
            std::env::var("ONUS_EXISTING_TEST_VALUE").as_deref(),
            Ok("keep")
        );
        assert_eq!(
            std::env::var("ONUS_NEW_TEST_VALUE").as_deref(),
            Ok("loaded")
        );
        assert_ne!(std::env::var("PATH").as_deref(), Ok("/tmp/not-loaded"));
        std::env::remove_var("ONUS_EXISTING_TEST_VALUE");
        std::env::remove_var("ONUS_NEW_TEST_VALUE");
    }
}
