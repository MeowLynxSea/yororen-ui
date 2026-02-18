use std::fs;
use std::path::{Path, PathBuf};

pub fn unique_child_path(parent: &Path, file_name: &str) -> PathBuf {
    // If dst exists, append " (n)".
    let mut candidate = parent.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.to_string());

    let ext = Path::new(file_name)
        .extension()
        .map(|e| e.to_string_lossy().to_string());

    for i in 1..=999u32 {
        let mut name = format!("{} ({})", stem, i);
        if let Some(ext) = &ext {
            name.push('.');
            name.push_str(ext);
        }
        candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    candidate
}

pub fn copy_path(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        copy_dir_recursive(src, dst)
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    let Ok(read_dir) = fs::read_dir(src) else {
        return Ok(());
    };

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(file_name);

        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

