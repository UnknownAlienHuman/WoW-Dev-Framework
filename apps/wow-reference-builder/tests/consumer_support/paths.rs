//! Project canonical test roots to consumer CLI paths without changing identity.
//! LuaLS treats Windows verbatim prefixes as URI authorities/glob characters.
use super::Result;
use std::path::{Path, PathBuf};

pub fn canonical_root(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()?;
    #[cfg(windows)]
    {
        let text = canonical.to_str().ok_or("non-UTF-8 consumer root")?;
        let projected = PathBuf::from(windows_path(text)?);
        // Removing a verbatim prefix must not change Win32 path semantics.
        if projected.canonicalize()? != canonical {
            return Err("consumer path projection changed the root identity".into());
        }
        Ok(projected)
    }
    #[cfg(not(windows))]
    Ok(canonical)
}

#[cfg(any(windows, test))]
fn windows_path(value: &str) -> Result<String> {
    if let Some(tail) = value.strip_prefix(r"\\?\UNC\") {
        let (server, rest) = tail.split_once('\\').ok_or("invalid UNC consumer root")?;
        let share = rest.split('\\').next().unwrap_or("");
        if server.is_empty() || share.is_empty() || matches!(server, "." | "?") {
            return Err("invalid UNC consumer root".into());
        }
        return Ok(format!(r"\\{tail}"));
    }
    let tail = value.strip_prefix(r"\\?\").unwrap_or(value);
    let bytes = tail.as_bytes();
    if bytes.len() < 3 || !bytes[0].is_ascii_alphabetic() || bytes[1..3] != *b":\\" {
        return Err("unsupported consumer root namespace".into());
    }
    Ok(tail.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_drive_and_unc_paths_without_changing_components() -> Result<()> {
        for (input, expected) in [
            (r"\\?\D:\a\_temp\probe", r"D:\a\_temp\probe"),
            (r"\\?\C:\space and Юникод\meta", r"C:\space and Юникод\meta"),
            (r"\\?\UNC\server\share\meta", r"\\server\share\meta"),
            (r"C:\plain\meta", r"C:\plain\meta"),
        ] {
            assert_eq!(windows_path(input)?, expected);
        }
        Ok(())
    }

    #[test]
    fn rejects_device_namespaces_and_drive_relative_paths() {
        for path in [
            r"\\?\GLOBALROOT\Device\HarddiskVolume1",
            r"\\?\Volume{example}\meta",
            r"\\.\C:\meta",
            r"\\?\C:meta",
            r"C:meta",
            r"\\?\UNC\\share\meta",
            r"\\?\UNC\server\",
        ] {
            assert!(windows_path(path).is_err(), "{path}");
        }
    }

    #[test]
    fn selected_workspace_keeps_the_same_canonical_identity() -> Result<()> {
        let workspace = super::super::Workspace::new()?;
        let projected = canonical_root(&workspace.path)?;
        assert_eq!(projected.canonicalize()?, workspace.path.canonicalize()?);
        #[cfg(windows)]
        assert!(!projected.to_string_lossy().starts_with(r"\\?\"));
        Ok(())
    }
}
