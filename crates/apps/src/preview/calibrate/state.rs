//! Radius, margin, and preview state for the calibration screen.

const DEFAULT_RADIUS: u16 = 64;
const DEFAULT_MARGIN: u16 = 12;
const MAX_RADIUS: u16 = 120;
const MAX_MARGIN: u16 = 48;
const READOUT_CAPACITY: usize = 64;

/// One calibration action selected by a tap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// Adjust the active corner radius by a signed step.
    Radius(i16),
    /// Adjust the optical margin by a signed step.
    Margin(i16),
    /// Cycle the active corner.
    Corner,
    /// Toggle the clipped-UI preview.
    Clip,
    /// Restore the defaults.
    Reset,
    /// Leave the calibration screen.
    Back,
}

/// Radius, margin, and preview state for the calibration screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CalibrateState {
    radii: [u16; 4],
    margin: u16,
    corner: Option<usize>,
    clip_ui: bool,
    version: u64,
    readout: [u8; READOUT_CAPACITY],
    readout_len: usize,
}

impl CalibrateState {
    /// Creates the calibration state with a deliberately generous radius.
    pub(crate) fn new() -> Self {
        let mut state = Self {
            radii: [DEFAULT_RADIUS; 4],
            margin: DEFAULT_MARGIN,
            corner: None,
            clip_ui: false,
            version: 0,
            readout: [0; READOUT_CAPACITY],
            readout_len: 0,
        };
        state.rewrite_readout();
        state
    }

    /// The per-corner radii in top-left, top-right, bottom-right, bottom-left order.
    pub(crate) const fn radii(&self) -> [u16; 4] {
        self.radii
    }

    /// The optical margin inside the panel shape.
    pub(crate) const fn margin(&self) -> u16 {
        self.margin
    }

    /// The active corner, or `None` for all corners.
    #[cfg(test)]
    pub(crate) const fn corner(&self) -> Option<usize> {
        self.corner
    }

    /// Whether the UI is masked to the candidate shape.
    pub(crate) const fn clip_ui(&self) -> bool {
        self.clip_ui
    }

    /// A value that changes whenever the shape changes.
    pub(crate) const fn fingerprint(&self) -> u64 {
        self.version
    }

    /// The formatted readout shown in the screen.
    pub(crate) fn readout(&self) -> &str {
        core::str::from_utf8(&self.readout[..self.readout_len]).unwrap_or("")
    }

    /// The active corner's short label.
    pub(crate) const fn corner_label(&self) -> &'static str {
        match self.corner {
            None => "All",
            Some(0) => "TL",
            Some(1) => "TR",
            Some(2) => "BR",
            Some(3) => "BL",
            Some(_) => "All",
        }
    }

    /// The clip-mode button label.
    pub(crate) const fn clip_label(&self) -> &'static str {
        if self.clip_ui {
            "Clip UI"
        } else {
            "Outline"
        }
    }

    /// Applies one action.
    pub(crate) fn apply(&mut self, action: Action) {
        match action {
            Action::Radius(delta) => self.adjust_radius(delta),
            Action::Margin(delta) => self.margin = adjust(self.margin, delta, MAX_MARGIN),
            Action::Corner => self.corner = next_corner(self.corner),
            Action::Clip => self.clip_ui = !self.clip_ui,
            Action::Reset => {
                self.radii = [DEFAULT_RADIUS; 4];
                self.margin = DEFAULT_MARGIN;
                self.corner = None;
                self.clip_ui = false;
            }
            Action::Back => return,
        }
        self.version = self.version.wrapping_add(1);
        self.rewrite_readout();
    }

    fn adjust_radius(&mut self, delta: i16) {
        if let Some(index) = self.corner {
            self.radii[index] = adjust(self.radii[index], delta, MAX_RADIUS);
        } else {
            self.radii = self.radii.map(|radius| adjust(radius, delta, MAX_RADIUS));
        }
    }

    fn selected_radius(&self) -> u16 {
        match self.corner {
            Some(index) if index < self.radii.len() => self.radii[index],
            _ => self.radii[0],
        }
    }

    fn rewrite_readout(&mut self) {
        let corner = self.corner_label();
        let selected = self.selected_radius();
        let radii = self.radii;
        let margin = self.margin;
        let mut writer = TextWriter::new(&mut self.readout);
        writer.push_str("R ");
        writer.push_u16(selected);
        writer.push_str("  M ");
        writer.push_u16(margin);
        writer.push_str("  ");
        writer.push_str(corner);
        writer.push_str("\nTL ");
        writer.push_u16(radii[0]);
        writer.push_str("  TR ");
        writer.push_u16(radii[1]);
        writer.push_str("  BR ");
        writer.push_u16(radii[2]);
        writer.push_str("  BL ");
        writer.push_u16(radii[3]);
        let len = writer.len();
        self.readout_len = len;
    }
}

fn adjust(value: u16, delta: i16, max: u16) -> u16 {
    let next = i32::from(value) + i32::from(delta);
    next.clamp(0, i32::from(max)) as u16
}

const fn next_corner(corner: Option<usize>) -> Option<usize> {
    match corner {
        None => Some(0),
        Some(0) => Some(1),
        Some(1) => Some(2),
        Some(2) => Some(3),
        Some(_) => None,
    }
}

struct TextWriter<'a> {
    bytes: &'a mut [u8],
    len: usize,
}

impl<'a> TextWriter<'a> {
    const fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes, len: 0 }
    }

    fn push_str(&mut self, text: &str) {
        for byte in text.as_bytes() {
            self.push_byte(*byte);
        }
    }

    fn push_u16(&mut self, value: u16) {
        let mut digits = [0u8; 5];
        let mut count = 0;
        let mut value = value;
        loop {
            digits[count] = b'0' + (value % 10) as u8;
            count += 1;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        while count > 0 {
            count -= 1;
            self.push_byte(digits[count]);
        }
    }

    fn push_byte(&mut self, byte: u8) {
        if self.len < self.bytes.len() {
            self.bytes[self.len] = byte;
            self.len += 1;
        }
    }

    const fn len(&self) -> usize {
        self.len
    }
}
