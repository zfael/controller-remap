#![cfg(windows)]

use controller_remap::domain::{PadState, BUTTON_Y};
use controller_remap::platform::vigem_writer::to_xgamepad;
use controller_remap::platform::xinput_reader::pad_from_state;

#[test]
fn xinput_state_maps_to_pad_state() {
    let mut state = rusty_xinput::XInputState::default();
    state.raw.Gamepad.wButtons = BUTTON_Y;
    state.raw.Gamepad.bLeftTrigger = 77;
    state.raw.Gamepad.sThumbLX = 123;

    let pad = pad_from_state(state);

    assert_eq!(pad.buttons, BUTTON_Y);
    assert_eq!(pad.left_trigger, 77);
    assert_eq!(pad.thumb_lx, 123);
}

#[test]
fn pad_state_maps_to_vigem_gamepad() {
    let pad = PadState {
        buttons: BUTTON_Y,
        left_trigger: 255,
        right_trigger: 0,
        thumb_lx: 50,
        thumb_ly: -50,
        thumb_rx: 0,
        thumb_ry: 0,
    };

    let gamepad = to_xgamepad(pad);

    assert_eq!(u16::from(gamepad.buttons), BUTTON_Y);
    assert_eq!(gamepad.left_trigger, 255);
    assert_eq!(gamepad.thumb_lx, 50);
    assert_eq!(gamepad.thumb_ly, -50);
}
