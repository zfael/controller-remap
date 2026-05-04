use anyhow::{anyhow, Context, Result};
use rusty_xinput::{XInputHandle, XInputState, XInputUsageError};

use crate::app::{InputFrame, InputSource};
use crate::domain::PadState;

pub struct XInputReader {
    handle: XInputHandle,
    slot: u32,
}

impl XInputReader {
    pub fn new(slot: u32) -> Result<Self> {
        let handle = XInputHandle::load_default().context("failed to load XInput")?;
        Ok(Self { handle, slot })
    }
}

pub fn pad_from_state(state: XInputState) -> PadState {
    PadState {
        buttons: state.raw.Gamepad.wButtons,
        left_trigger: state.left_trigger(),
        right_trigger: state.right_trigger(),
        thumb_lx: state.raw.Gamepad.sThumbLX,
        thumb_ly: state.raw.Gamepad.sThumbLY,
        thumb_rx: state.raw.Gamepad.sThumbRX,
        thumb_ry: state.raw.Gamepad.sThumbRY,
    }
}

impl InputSource for XInputReader {
    fn poll(&mut self) -> Result<InputFrame> {
        match self.handle.get_state(self.slot) {
            Ok(state) => Ok(InputFrame {
                connected: true,
                state: pad_from_state(state),
            }),
            Err(XInputUsageError::DeviceNotConnected) => Ok(InputFrame {
                connected: false,
                state: PadState::neutral(),
            }),
            Err(err) => Err(anyhow!("xinput read failed: {err:?}")),
        }
    }
}
