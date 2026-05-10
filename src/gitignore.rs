use std::path::Path;

use crate::error::Result;

pub fn ensure_entries_in_gitignore(dir: &Path, entries: &[&str]) -> Result<()> {
    let path = dir.join(".gitignore");

    if !crate::io::exists(&path)? {
        let content = entries.iter().map(|e| format!("{e}\n")).collect::<String>();
        crate::io::write(&path, content)?;
        return Ok(());
    }

    let existing = crate::io::read_to_string(&path)?;
    let mut updated = existing.clone();

    let mut added_anything = false;
    for entry in entries {
        if !has_entry(&updated, entry) {
            if !updated.is_empty() && !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(entry);
            updated.push('\n');
            added_anything = true;
        }
    }

    if added_anything {
        crate::io::write(&path, updated)?;
    }
    Ok(())
}

fn has_entry(content: &str, entry: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        // skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return false;
        }
        // strip leading slash and trailing slash for comparison
        // so "target", "/target", "target/", "/target/" all match
        let normalized = trimmed.trim_start_matches('/').trim_end_matches('/');
        normalized == entry
    })
}
