use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::config::Config;

pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

pub fn import_legacy(path: &Path, config: &mut Config) -> Result<ImportResult> {
    let text =
        fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {}: {}", path.display(), e))?;
    let mut imported = 0;
    let mut skipped = 0;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, ':');
        let name = match parts.next() {
            Some(n) => n.trim().to_string(),
            None => continue,
        };
        let rest = match parts.next() {
            Some(r) => r,
            None => continue,
        };
        let dirs: Vec<String> = rest.split(':').map(|d| d.trim().to_string()).collect();
        match config.add(name, dirs) {
            Ok(()) => imported += 1,
            Err(_) => skipped += 1,
        }
    }
    Ok(ImportResult { imported, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn import_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".mytmux");
        fs::write(
            &path,
            "# comment\n\nmyproject:/home/user/proj\ninfra:/home/user/infra:/home/user/infra/tf\n",
        )
        .unwrap();
        let mut cfg = Config::default();
        let result = import_legacy(&path, &mut cfg).unwrap();
        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped, 0);
        assert_eq!(cfg.session[1].dirs.len(), 2);
    }

    #[test]
    fn import_skips_duplicates() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".mytmux");
        fs::write(&path, "proj:/home/user/proj\n").unwrap();
        let mut cfg = Config::default();
        cfg.add("proj".into(), vec!["/other".into()]).unwrap();
        let result = import_legacy(&path, &mut cfg).unwrap();
        assert_eq!(result.imported, 0);
        assert_eq!(result.skipped, 1);
    }
}
