use tempfile::tempdir;

use crate::NostrMls;

pub fn create_test_nostr_mls() -> NostrMls {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    NostrMls::new(temp_dir.path().to_path_buf(), None)
}
