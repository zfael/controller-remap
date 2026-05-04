pub const BUTTON_A: u16 = 0x1000;
pub const BUTTON_Y: u16 = 0x8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PadState {
    pub buttons: u16,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub thumb_lx: i16,
    pub thumb_ly: i16,
    pub thumb_rx: i16,
    pub thumb_ry: i16,
}

impl PadState {
    pub const fn neutral() -> Self {
        Self {
            buttons: 0,
            left_trigger: 0,
            right_trigger: 0,
            thumb_lx: 0,
            thumb_ly: 0,
            thumb_rx: 0,
            thumb_ry: 0,
        }
    }

    pub fn without_buttons(mut self, mask: u16) -> Self {
        self.buttons &= !mask;
        self
    }
}
