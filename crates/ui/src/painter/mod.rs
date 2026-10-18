//! Clipped drawing primitives.
//!
//! The painter owns a small clip stack and talks to any [`Surface`]. It never
//! allocates and never draws outside the active clip.

use common::Region;

use crate::luma::Luma;
use crate::shape::ScreenShape;
use crate::surface::Surface;

mod geometry;

use geometry::{fill_shaped_rows, inside_rounded, offset_axis};

/// Maximum nested clips.
pub const MAX_CLIP_DEPTH: usize = 8;

/// A clipped drawing context.
pub struct Painter<'a> {
    surface: &'a mut dyn Surface,
    clips: [Region; MAX_CLIP_DEPTH],
    depth: usize,
    shape: Option<ScreenShape>,
}

impl<'a> Painter<'a> {
    /// Starts a painter with the full surface as the root clip.
    pub fn new(surface: &'a mut dyn Surface) -> Self {
        let clip = Region::full(surface.width(), surface.height());
        Self {
            surface,
            clips: [clip; MAX_CLIP_DEPTH],
            depth: 1,
            shape: None,
        }
    }

    /// Constrains every draw to the panel silhouette.
    pub fn set_shape(&mut self, shape: Option<ScreenShape>) {
        self.shape = shape;
    }

    /// The active panel silhouette.
    pub const fn shape(&self) -> Option<ScreenShape> {
        self.shape
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
        let Some(shape) = self.shape else {
            self.surface.fill_rect(region, color);
            return;
        };
        let width = self.surface.width();
        let height = self.surface.height();
        let radius = shape.corner_radius.min(width.min(height) / 2);
        let middle_start = region.y.max(radius);
        let middle_end = region.bottom().min(height.saturating_sub(radius));
        let top_end = region.bottom().min(middle_start);
        fill_shaped_rows(self.surface, shape, region, region.y..top_end, color);
        if middle_start < middle_end {
            self.surface.fill_rect(
                Region::new(
                    region.x,
                    middle_start,
                    region.width,
                    middle_end - middle_start,
                ),
                color,
            );
        }
        let bottom_start = region.y.max(middle_end);
        fill_shaped_rows(
            self.surface,
            shape,
            region,
            bottom_start..region.bottom(),
            color,
        );
    }

    /// Sets one pixel, clipped to the active clip.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Luma) {
        if self.clip().contains(x, y) && self.inside_shape(x, y) {
            self.surface.set_pixel(x, y, color);
        }
    }

    /// Blends one pixel with 4-bit coverage, clipped to the active clip.
    pub fn blend_pixel(&mut self, x: u16, y: u16, color: Luma, coverage: u8) {
        let coverage = coverage.min(15);
        if coverage == 0 || !self.clip().contains(x, y) || !self.inside_shape(x, y) {
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

    /// Draws one quadrant of a one-pixel circle.
    pub fn arc_quadrant(
        &mut self,
        center: (u16, u16),
        radius: u16,
        is_right: bool,
        is_bottom: bool,
        color: Luma,
    ) {
        if radius == 0 {
            return;
        }
        let (center_x, center_y) = center;
        let squared = u32::from(radius) * u32::from(radius);
        for offset in 0..=u32::from(radius) {
            let other = crate::shape::integer_sqrt(squared - offset * offset) as u16;
            let offset = offset as u16;
            let x = offset_axis(center_x, offset, is_right);
            let y = offset_axis(center_y, other, is_bottom);
            self.set_pixel(x, y, color);
            let x = offset_axis(center_x, other, is_right);
            let y = offset_axis(center_y, offset, is_bottom);
            self.set_pixel(x, y, color);
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

    fn inside_shape(&self, x: u16, y: u16) -> bool {
        match self.shape {
            Some(shape) => shape.contains(x, y, self.surface.width(), self.surface.height()),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests;
