//! Shared-secret token that both local IPC transports (`SocketAdapter`,
//! `WebSocketAdapter`) require on every message, so an unauthenticated local
//! process or a malicious webpage can't drive a target's native commands.
//! See docs/protocol.md#trust-model--authentication for the wire-level
//! contract and threat model this defends against.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

const TOKEN_FILE_NAME: &str = "secret.token";
const TOKEN_BYTES: usize = 32;

/// Loads the per-install shared secret from `<config_dir>/secret.token`,
/// generating and persisting a new one on first run. Reusing an existing
/// file means restarting the daemon never invalidates already-paired
/// extensions.
pub fn load_or_create_token(config_dir: &Path) -> io::Result<String> {
    let path = config_dir.join(TOKEN_FILE_NAME);
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(contents.trim().to_string()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let token = generate_token();
            write_token_file(&path, &token)?;
            Ok(token)
        }
        Err(e) => Err(e),
    }
}

fn generate_token() -> String {
    use rand::Rng;
    let mut bytes = [0u8; TOKEN_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    hex_encode(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(unix)]
fn write_token_file(path: &Path, token: &str) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    // `create_new` + mode 0o600 in one syscall avoids a write-then-chmod
    // window where the token would briefly be world-readable.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(token.as_bytes())
}

#[cfg(not(unix))]
fn write_token_file(path: &Path, token: &str) -> io::Result<()> {
    // No POSIX mode bits on Windows; default NTFS ACLs (owner + admins) are
    // the accepted trust boundary here, same as any other per-user secret
    // file (e.g. an SSH private key).
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(token.as_bytes())
}

/// Reads the token back out of an already-open file handle, used only by
/// tests to check what `write_token_file` actually persisted.
#[cfg(test)]
fn read_token_file(path: &Path) -> io::Result<String> {
    use std::io::Read;
    let mut contents = String::new();
    std::fs::File::open(path)?.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "keylex-auth-test-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn generates_a_64_char_hex_token_on_first_run() {
        let dir = temp_dir("first-run");
        let token = load_or_create_token(&dir).expect("should generate a token");
        assert_eq!(token.len(), TOKEN_BYTES * 2);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reuses_existing_token_across_calls() {
        let dir = temp_dir("reuse");
        let first = load_or_create_token(&dir).expect("first load creates token");
        let second = load_or_create_token(&dir).expect("second load reuses token");
        assert_eq!(first, second);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn token_file_is_created_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir("perms");
        load_or_create_token(&dir).expect("should generate a token");
        let meta = std::fs::metadata(dir.join(TOKEN_FILE_NAME)).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stored_token_round_trips_through_the_file() {
        let dir = temp_dir("roundtrip");
        let token = load_or_create_token(&dir).expect("should generate a token");
        let on_disk = read_token_file(&dir.join(TOKEN_FILE_NAME)).unwrap();
        assert_eq!(on_disk.trim(), token);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
