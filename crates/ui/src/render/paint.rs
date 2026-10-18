//! Damage repair painting.

use common::Region;

use crate::luma::Luma;
use crate::painter::Painter;
use crate::scene::Damage;
use crate::surface::Surface;

/// Repaints every dirty region, clipped to that region.
///
/// The callback draws the current scene once per dirty region. The painter is
/// already clipped, so drawing may safely ignore the damage bounds.
pub fn paint_damage<F>(surface: &mut dyn Surface, damage: &Damage, background: Luma, mut draw: F)
where
    F: FnMut(&mut Painter<'_>, Region),
{
    let mut painter = Painter::new(surface);
    for region in damage.regions() {
        if !painter.push_clip(*region) {
            continue;
        }
        painter.clear(background);
        draw(&mut painter, *region);
        painter.pop_clip();
    }
}
