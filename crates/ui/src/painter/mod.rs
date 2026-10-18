//! Clipped drawing primitives.
//!
//! The painter owns a small clip stack and talks to any [`Surface`]. It never
//! allocates and never draws outside the active clip.

use common::Region;

use crate::luma::Luma;
use crate::surface::Surface;

/// Maximum nested clips.
pub const MAX_CLIP_DEPTH: usize = 8;

/// A clipped drawing context.
pub struct Painter<'a> {
    surface: &'a mut dyn Surface,
    clips: [Region; MAX_CLIP_DEPTH],
    depth: usize,
}

impl<'a> Painter<'a> {
    /// Starts a painter with the full surface as the root clip.
    pub fn new(surface: &'a mut dyn Surface) -> Self {
        let clip = Region::full(surface.width(), surface.height());
        Self {
            surface,
            clips: [clip; MAX_CLIP_DEPTH],
            depth: 1,
        }
    }

    /// The surface width in pixels.
    pub fn width(&self) -> u16 {
        self.surface.width()
    }

    /// The surface height in pixels.
    pub fn height(&self) -> u16 {
        self.surface.height()
    }

    /// The active clip region.
    pub fn clip(&self) -> Region {
        self.clips[self.depth - 1]
    }

    /// Intersects `region` with the active clip and pushes it.
    ///
    /// Returns `false` when the stack is full or the clip is empty.
    pub fn push_clip(&mut self, region: Region) -> bool {
        if self.depth == MAX_CLIP_DEPTH {
            return false;
        }
        let clip = region.intersection(self.clip());
        if clip.is_empty() {
            return false;
        }
        self.clips[self.depth] = clip;
        self.depth += 1;
        true
    }

    /// Pops the active clip. The root clip cannot be popped.
    pub fn pop_clip(&mut self) -> bool {
        if self.depth <= 1 {
            return false;
        }
        self.depth -= 1;
        true
    }

    /// Fills the active clip.
    pub fn clear(&mut self, color: Luma) {
        self.fill_rect(self.clip(), color);
    }

    /// Fills a region, clipped to the active clip.
    pub fn fill_rect(&mut self, region: Region, color: Luma) {
        let region = region.intersection(self.clip());
        if region.is_empty() {
            return;
        }
        self.surface.fill_rect(region, color);
    }

    /// Sets one pixel, clipped to the active clip.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Luma) {
        if self.clip().contains(x, y) {
            self.surface.set_pixel(x, y, color);
        }
    }

    /// Blends one pixel with 4-bit coverage, clipped to the active clip.
    pub fn blend_pixel(&mut self, x: u16, y: u16, color: Luma, coverage: u8) {
        let coverage = coverage.min(15);
        if coverage == 0 || !self.clip().contains(x, y) {
            return;
        }
        if coverage == 15 {
            self.surface.set_pixel(x, y, color);
            return;
        }
        let foreground = u16::from(color.raw());
        let background = u16::from(self.surface.pixel(x, y).raw());
        let alpha = u16::from(coverage);
        let value = (foreground * alpha + background * (15 - alpha) + 7) / 15;
        self.surface.set_pixel(x, y, Luma::new(value as u8));
    }

    /// Draws a rectangular outline inside `region`.
    pub fn stroke_rect(&mut self, region: Region, thickness: u16, color: Luma) {
        if region.is_empty() || thickness == 0 {
            return;
        }
        let thickness = thickness.min(region.width.min(region.height));
        let doubled = thickness.saturating_mul(2);
        if region.width <= doubled || region.height <= doubled {
            self.fill_rect(region, color);
            return;
        }
        self.fill_rect(
            Region::new(region.x, region.y, region.width, thickness),
            color,
        );
        self.fill_rect(
            Region::new(
                region.x,
                region.bottom() - thickness,
                region.width,
                thickness,
            ),
            color,
        );
        self.fill_rect(
            Region::new(
                region.x,
                region.y + thickness,
                thickness,
                region.height - doubled,
            ),
            color,
        );
        self.fill_rect(
            Region::new(
                region.right() - thickness,
                region.y + thickness,
                thickness,
                region.height - doubled,
            ),
            color,
        );
    }

    /// Fills a rounded rectangle with hard-edged corners.
    pub fn fill_rounded_rect(&mut self, region: Region, radius: u16, color: Luma) {
        if region.is_empty() {
            return;
        }
        let radius = radius.min(region.width.min(region.height) / 2);
        if radius == 0 {
            self.fill_rect(region, color);
            return;
        }
        let clipped = region.intersection(self.clip());
        for y in clipped.y..clipped.bottom() {
            for x in clipped.x..clipped.right() {
                if inside_rounded(region, radius, x, y) {
                    self.surface.set_pixel(x, y, color);
                }
            }
        }
    }

    /// Draws a one-pixel line.
    pub fn line(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: Luma) {
        let mut x = i32::from(x0);
        let mut y = i32::from(y0);
        let target_x = i32::from(x1);
        let target_y = i32::from(y1);
        let dx = (target_x - x).abs();
        let dy = -(target_y - y).abs();
        let step_x = if x < target_x { 1 } else { -1 };
        let step_y = if y < target_y { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.set_pixel(x as u16, y as u16, color);
            if x == target_x && y == target_y {
                break;
            }
            let doubled = error * 2;
            if doubled >= dy {
                error += dy;
                x += step_x;
            }
            if doubled <= dx {
                error += dx;
                y += step_y;
            }
        }
    }
}

fn inside_rounded(region: Region, radius: u16, x: u16, y: u16) -> bool {
    let left = i32::from(region.x);
    let top = i32::from(region.y);
    let right = i32::from(region.right());
    let bottom = i32::from(region.bottom());
    let radius = i32::from(radius);
    let x = i32::from(x);
    let y = i32::from(y);
    let center_x = if x < left + radius {
        left + radius
    } else if x >= right - radius {
        right - radius
    } else {
        return true;
    };
    let center_y = if y < top + radius {
        top + radius
    } else if y >= bottom - radius {
        bottom - radius
    } else {
        return true;
    };
    let dx = (x * 2 + 1) - center_x * 2;
    let dy = (y * 2 + 1) - center_y * 2;
    dx * dx + dy * dy <= (radius * 2) * (radius * 2)
}

#[cfg(test)]
mod tests;
