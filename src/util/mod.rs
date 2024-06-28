use std::{ffi::OsStr, path::Path};

pub mod fs;
pub mod sem;
pub mod token;

pub fn resolve_osstr(osstr: Option<&OsStr>) -> Result<&str, Box<dyn std::error::Error + Send + Sync>> {
    Ok(osstr.ok_or("resolve osstr failed")?.to_str().ok_or("osstr to str error")?)
}
pub fn resolve_osstr_default(osstr: Option<&OsStr>) -> Result<&str, Box<dyn std::error::Error + Send + Sync>> {
    match osstr {
        Some(osstr) => Ok(osstr.to_str().ok_or("osstr to str error")?),
        None => Ok(""),
    }
}

pub fn resolve_path(path: &Path) -> Result<&str, Box<dyn std::error::Error + Send + Sync>> {
    Ok(path.to_str().ok_or("resolve path failed")?)
}

