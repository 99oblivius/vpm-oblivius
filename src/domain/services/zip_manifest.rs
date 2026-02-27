use std::io::Read;
use std::path::Path;

use crate::app_error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct ZipManifest {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub full_json: serde_json::Value,
}

/// Maximum size of package.json we'll read from the zip (1 MB).
const MAX_MANIFEST_SIZE: u64 = 1_048_576;

/// Validates that a package name is filesystem-safe.
fn validate_package_name(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(AppError::BadRequest("package.json `name` is empty".into()));
    }
    let re_ok = name.bytes().all(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'-' | b'_'));
    if !re_ok || !name.as_bytes()[0].is_ascii_alphanumeric() {
        return Err(AppError::BadRequest(
            "package.json `name` must match ^[a-zA-Z0-9][a-zA-Z0-9._-]*$".into(),
        ));
    }
    Ok(())
}

/// Extract `package.json` from a zip file at the given path.
/// Requires `package.json` at the zip root — VCC/ALCOM expects flat zip structure.
pub fn extract_manifest(path: &Path) -> AppResult<ZipManifest> {
    let file = std::fs::File::open(path)
        .map_err(|e| AppError::BadRequest(format!("Failed to open zip: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::BadRequest(format!("Invalid zip archive: {e}")))?;

    let manifest_index = (0..archive.len())
        .find(|&i| {
            if let Ok(entry) = archive.by_index(i) {
                entry.name() == "package.json"
            } else {
                false
            }
        })
        .ok_or_else(|| AppError::BadRequest(
            "package.json must be at the zip root (not inside a subdirectory)".into()
        ))?;

    let mut entry = archive.by_index(manifest_index)
        .map_err(|e| AppError::BadRequest(format!("Failed to read zip entry: {e}")))?;

    if entry.size() > MAX_MANIFEST_SIZE {
        return Err(AppError::BadRequest("package.json exceeds 1 MB".into()));
    }

    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut buf)
        .map_err(|e| AppError::BadRequest(format!("Failed to read package.json from zip: {e}")))?;

    let json: serde_json::Value = serde_json::from_slice(&buf)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON in package.json: {e}")))?;

    let name = json.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("package.json missing required field `name`".into()))?
        .to_string();

    let display_name = json.get("displayName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("package.json missing required field `displayName`".into()))?
        .to_string();

    let version = json.get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("package.json missing required field `version`".into()))?
        .to_string();

    // Validate semver
    semver::Version::parse(&version)
        .map_err(|e| AppError::BadRequest(format!("package.json `version` is not valid semver: {e}")))?;

    validate_package_name(&name)?;

    Ok(ZipManifest {
        name,
        display_name,
        version,
        full_json: json,
    })
}
