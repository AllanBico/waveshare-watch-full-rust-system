//! Asset manifest parsing.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// The root manifest.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// Font sources.
    #[serde(default)]
    pub font: Vec<FontSpec>,
    /// Icon sources.
    pub icons: IconsSpec,
}

/// One font source and the sizes to generate from it.
#[derive(Debug, Deserialize)]
pub struct FontSpec {
    /// `ui` or `numeric`.
    pub id: String,
    /// `regular`, `medium`, or `semibold`.
    pub weight: String,
    /// Path relative to `assets/`.
    pub path: String,
    /// Pixel sizes to generate.
    pub sizes: Vec<u16>,
    /// `ui` or `numeric`.
    pub charset: String,
}

/// Icon generation settings.
#[derive(Debug, Deserialize)]
pub struct IconsSpec {
    /// Pixel sizes to generate.
    pub sizes: Vec<u16>,
    /// Selected icons.
    #[serde(default)]
    pub item: Vec<IconSpec>,
}

/// One selected icon.
#[derive(Debug, Deserialize)]
pub struct IconSpec {
    /// Logical name, converted to an `IconId` variant.
    pub id: String,
    /// Path relative to `assets/`.
    pub file: String,
}

impl Manifest {
    /// Reads and parses a manifest.
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("read manifest {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parse manifest {}", path.display()))
    }
}
