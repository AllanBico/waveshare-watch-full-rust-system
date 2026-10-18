//! Compiled-in icon rendering.

use crate::assets::{IconId, ICON_SETS};
use crate::luma::Luma;
use crate::painter::Painter;

/// Draws a square icon with its top-left at `origin`.
pub fn draw_icon(
    painter: &mut Painter<'_>,
    icon: IconId,
    size_px: u16,
    origin: (u16, u16),
    color: Luma,
) {
    let Some(set) = ICON_SETS.iter().find(|set| set.size_px == size_px) else {
        return;
    };
    let Some(mask) = set.masks.get(icon as usize) else {
        return;
    };
    let Some(data) = mask_data(set.data, mask.offset, mask.len) else {
        return;
    };
    for row in 0..mask.height {
        for col in 0..mask.width {
            let index = usize::from(row) * usize::from(mask.width) + usize::from(col);
            let Some(coverage) = nibble(data, index) else {
                continue;
            };
            painter.blend_pixel(
                origin.0.saturating_add(col),
                origin.1.saturating_add(row),
                color,
                coverage,
            );
        }
    }
}

fn mask_data(data: &[u8], offset: u32, len: u32) -> Option<&[u8]> {
    let start = offset as usize;
    let end = start.checked_add(len as usize)?;
    data.get(start..end)
}

fn nibble(mask: &[u8], index: usize) -> Option<u8> {
    let byte = *mask.get(index / 2)?;
    Some(if index.is_multiple_of(2) {
        byte >> 4
    } else {
        byte & 0x0F
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::MonoSurface;

    #[test]
    fn icons_light_pixels_at_generated_sizes() {
        let mut data = vec![0u8; MonoSurface::buffer_len(24, 24)];
        let mut surface = MonoSurface::new(24, 24, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            draw_icon(&mut painter, IconId::Clock, 24, (0, 0), Luma::WHITE);
        }
        assert!(surface.lit_pixels() > 0);
    }

    #[test]
    fn unknown_size_draws_nothing() {
        let mut data = vec![0u8; MonoSurface::buffer_len(24, 24)];
        let mut surface = MonoSurface::new(24, 24, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            draw_icon(&mut painter, IconId::Clock, 17, (0, 0), Luma::WHITE);
        }
        assert_eq!(surface.lit_pixels(), 0);
    }
}
