//! Damage flush to an RGB565 display.

use common::Region;
use hal::Display;

use crate::scene::Damage;
use crate::surface::{GraySurface, MonoSurface, Surface};

/// A surface that can expand a region to big-endian RGB565 bytes.
pub trait Rgb565Source: Surface {
    /// Writes `min(out.len(), region.rgb565_len())` bytes.
    fn expand_rgb565(&self, region: Region, out: &mut [u8]);
}

impl Rgb565Source for MonoSurface<'_> {
    fn expand_rgb565(&self, region: Region, out: &mut [u8]) {
        MonoSurface::expand_region(self, region, out);
    }
}

impl Rgb565Source for GraySurface<'_> {
    fn expand_rgb565(&self, region: Region, out: &mut [u8]) {
        GraySurface::expand_region(self, region, out);
    }
}

/// A flush failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlushError<E> {
    /// The display rejected a write.
    Display(E),
    /// The staging buffer cannot hold even one aligned two-row strip.
    StagingTooSmall,
}

/// Pushes all dirty regions to the display, one TE-synced frame.
///
/// Regions are aligned outward to even coordinates and extents because the
/// CO5300 address window requires it. Each region is expanded in horizontal
/// strips that fit `staging`.
pub fn flush<D: Display>(
    display: &mut D,
    source: &dyn Rgb565Source,
    damage: &Damage,
    staging: &mut [u8],
) -> Result<(), FlushError<D::Error>> {
    if damage.is_empty() {
        return Ok(());
    }
    display.wait_for_vsync().map_err(FlushError::Display)?;
    for region in damage.regions() {
        let region = align_even(*region, source.width(), source.height());
        if region.is_empty() {
            continue;
        }
        flush_region(display, source, region, staging)?;
    }
    Ok(())
}

fn align_even(region: Region, width: u16, height: u16) -> Region {
    if region.is_empty() {
        return Region::new(0, 0, 0, 0);
    }
    let left = region.x & !1u16;
    let top = region.y & !1u16;
    let right = (region.right().saturating_add(1) & !1u16).min(width);
    let bottom = (region.bottom().saturating_add(1) & !1u16).min(height);
    if right <= left || bottom <= top {
        return Region::new(left, top, 0, 0);
    }
    Region::new(left, top, right - left, bottom - top)
}

fn flush_region<D: Display>(
    display: &mut D,
    source: &dyn Rgb565Source,
    region: Region,
    staging: &mut [u8],
) -> Result<(), FlushError<D::Error>> {
    let row_bytes = usize::from(region.width) * 2;
    let rows = (staging.len() / row_bytes) & !1;
    if rows == 0 {
        return Err(FlushError::StagingTooSmall);
    }
    let rows = rows.min(usize::from(region.height));
    let mut y = region.y;
    while y < region.bottom() {
        let remaining = usize::from(region.bottom() - y);
        let height = rows.min(remaining) as u16;
        let strip = Region::new(region.x, y, region.width, height);
        let bytes = strip.rgb565_len();
        source.expand_rgb565(strip, &mut staging[..bytes]);
        display
            .write_region(strip, &staging[..bytes])
            .map_err(FlushError::Display)?;
        y += height;
    }
    Ok(())
}
