//! Shared output and naming helpers.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// One generated file.
pub struct Output {
    /// Path relative to the repository root.
    pub path: PathBuf,
    /// File bytes.
    pub bytes: Vec<u8>,
}

/// Converts a manifest id into a PascalCase enum variant.
pub fn pascal_case(id: &str) -> String {
    let mut out = String::new();
    for part in id.split(['_', '-']).filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    out
}

/// Maps a manifest font id to a `FontId` expression.
pub fn font_id_expr(id: &str) -> Result<&'static str> {
    match id {
        "ui" => Ok("crate::theme::FontId::Ui"),
        "numeric" => Ok("crate::theme::FontId::Numeric"),
        other => bail!("unknown font id {other}"),
    }
}

/// Maps a manifest weight to a `Weight` expression.
pub fn weight_expr(weight: &str) -> Result<&'static str> {
    match weight {
        "regular" => Ok("crate::theme::Weight::Regular"),
        "medium" => Ok("crate::theme::Weight::Medium"),
        "semibold" => Ok("crate::theme::Weight::Semibold"),
        other => bail!("unknown weight {other}"),
    }
}

/// Writes all outputs, replacing the generated trees.
pub fn write_all(root: &Path, outputs: &[Output]) -> Result<()> {
    for dir in [
        "crates/ui/assets/fonts",
        "crates/ui/assets/icons",
        "crates/ui/src/assets",
    ] {
        let path = root.join(dir);
        if path.exists() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("remove {}", path.display()))?;
        }
    }
    for output in outputs {
        let path = root.join(&output.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&path, &output.bytes)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

/// Verifies that every output already matches.
pub fn check_all(root: &Path, outputs: &[Output]) -> Result<()> {
    let mut stale = Vec::new();
    for output in outputs {
        let path = root.join(&output.path);
        match fs::read(&path) {
            Ok(bytes) if bytes == output.bytes => {}
            Ok(_) => stale.push(format!("changed: {}", output.path.display())),
            Err(_) => stale.push(format!("missing: {}", output.path.display())),
        }
    }
    if stale.is_empty() {
        return Ok(());
    }
    for line in &stale {
        eprintln!("{line}");
    }
    bail!("generated assets are stale; run assetgen")
}

/// Prints generated sizes and enforces the per-asset limit.
pub fn report_sizes(outputs: &[Output]) -> Result<()> {
    const LIMIT: usize = 64 * 1024;
    let mut total = 0usize;
    for output in outputs {
        let len = output.bytes.len();
        total += len;
        println!("{:>7}  {}", len, output.path.display());
        if output.path.extension().is_some_and(|ext| ext == "bin") && len > LIMIT {
            bail!(
                "{} is {} bytes, over the {} byte limit",
                output.path.display(),
                len,
                LIMIT
            );
        }
    }
    println!("{:>7}  total", total);
    Ok(())
}
