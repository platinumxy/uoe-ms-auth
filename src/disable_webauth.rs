use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

const MANIFEST_JSON: &str = r#"{
  "manifest_version": 3,
  "name": "Disable WebAuthn API",
  "version": "1.0",
  "description": "Disables navigator.credentials WebAuthn for testing",
  "content_scripts": [
    {
      "matches": ["<all_urls>"],
      "js": ["disable_webauthn.js"],
      "run_at": "document_start",
      "world": "MAIN"
    }
  ]
}"#;

const DISABLE_WEBAUTHN_JS: &str = r#"
(function() {
  'use strict';
  try {
    Object.defineProperty(navigator, 'credentials', {
      get: () => undefined,
      set: () => undefined,
      configurable: false
    });
  } catch (e) {
    console.warn('[WebAuthn Disabler] Failed:', e.message);
  }
})();
"#;

static TEMP_DIR: Mutex<Option<Arc<TempDir>>> = Mutex::new(None);

pub fn get_extension_path() -> Result<(PathBuf, Arc<TempDir>), Box<dyn std::error::Error>> {
    let mut temp_dir_lock = TEMP_DIR.lock().unwrap();

    if let Some(arc_tempdir) = temp_dir_lock.as_ref() {
        let path = arc_tempdir.path().to_path_buf();
        return Ok((path, Arc::clone(arc_tempdir)));
    }

    let temp_dir = TempDir::new()?;
    let ext_path = temp_dir.path();

    std::fs::write(ext_path.join("manifest.json"), MANIFEST_JSON)?;
    std::fs::write(ext_path.join("disable_webauthn.js"), DISABLE_WEBAUTHN_JS)?;

    let path = ext_path.to_path_buf();
    let arc_tempdir = Arc::new(temp_dir);
    *temp_dir_lock = Some(Arc::clone(&arc_tempdir));

    Ok((path, arc_tempdir))
}
