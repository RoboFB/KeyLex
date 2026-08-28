//! The shared secret both local IPC transports require on every message, so
//! that an unauthenticated local process or a malicious web page can't
//! drive a target's native commands. See
//! `docs/protocol.md#trust-model--authentication` for the wire-level
//! contract and the threat model it defends against.

use std::fs::OpenOptions;
use std::io::{self, Write as _};
use std::path::Path;

const TOKEN_FILE_NAME: &str = "secret.token";
const TOKEN_BYTES: usize = 32;

/// Loads the per-install secret, generating and persisting one on first
/// run. Reusing the existing file is what keeps a daemon restart from
/// unpairing every extension.
pub fn load_or_create_token(config_dir: &Path) -> io::Result<String> {
    let path = config_dir.join(TOKEN_FILE_NAME);
    match std::fs::read_to_string(&path) {
        Ok(token) => Ok(token.trim().to_string()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let token = generate_token();
            write_token(&path, &token)?;
            Ok(token)
        }
        Err(e) => Err(e),
    }
}

fn generate_token() -> String {
    use rand::Rng as _;

    let mut bytes = [0u8; TOKEN_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(unix)]
fn write_token(path: &Path, token: &str) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt as _;

    // `create_new` plus mode 0o600 in one syscall: a write-then-chmod would
    // leave a window where the token is world-readable.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?
        .write_all(token.as_bytes())
}

#[cfg(not(unix))]
fn write_token(path: &Path, token: &str) -> io::Result<()> {
    // No POSIX mode bits here; the default NTFS ACL (owner plus admins) is
    // the accepted trust boundary, as it is for any other per-user secret.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(token.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("keylex-auth-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn first_run_generates_a_hex_token_and_later_runs_reuse_it() {
        let dir = temp_dir("reuse");
        let token = load_or_create_token(&dir).unwrap();

        assert_eq!(token.len(), TOKEN_BYTES * 2);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(load_or_create_token(&dir).unwrap(), token);
        assert_eq!(
            std::fs::read_to_string(dir.join(TOKEN_FILE_NAME)).unwrap(),
            token
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn the_token_file_is_readable_only_by_its_owner() {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = temp_dir("perms");
        load_or_create_token(&dir).unwrap();
        let mode = std::fs::metadata(dir.join(TOKEN_FILE_NAME))
            .unwrap()
            .permissions()
            .mode();

        assert_eq!(mode & 0o777, 0o600);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
