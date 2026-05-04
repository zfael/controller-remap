use std::time::{Duration, Instant};

use controller_remap::domain::{MacroEngine, PadState, BUTTON_A, BUTTON_Y};

fn engine() -> MacroEngine {
    MacroEngine::new(Duration::from_secs(1), Duration::from_millis(60))
}

fn neutral() -> PadState {
    PadState::neutral()
}

#[test]
fn physical_y_toggles_looping_only_on_rising_edge() {
    let start = Instant::now();
    let mut engine = engine();

    let first = engine.process_frame(
        start,
        true,
        PadState {
            buttons: BUTTON_Y,
            ..neutral()
        },
    );
    assert!(engine.is_looping());
    assert_eq!(first.buttons & BUTTON_Y, BUTTON_Y);

    let held = engine.process_frame(
        start + Duration::from_millis(20),
        true,
        PadState {
            buttons: BUTTON_Y,
            ..neutral()
        },
    );
    assert!(engine.is_looping());
    assert_eq!(held.buttons & BUTTON_Y, BUTTON_Y);

    engine.process_frame(start + Duration::from_millis(80), true, neutral());
    engine.process_frame(
        start + Duration::from_millis(100),
        true,
        PadState {
            buttons: BUTTON_Y,
            ..neutral()
        },
    );
    assert!(!engine.is_looping());
}

#[test]
fn idle_mode_passes_non_y_buttons_through_unchanged() {
    let start = Instant::now();
    let mut engine = engine();

    let output = engine.process_frame(
        start,
        true,
        PadState {
            buttons: BUTTON_A,
            ..neutral()
        },
    );
    assert_eq!(output.buttons & BUTTON_Y, 0);
    assert_eq!(output.buttons & BUTTON_A, BUTTON_A);
}

#[test]
fn looping_emits_y_and_lt_on_a_fixed_cadence() {
    let start = Instant::now();
    let mut engine = engine();

    engine.process_frame(
        start,
        true,
        PadState {
            buttons: BUTTON_Y,
            ..neutral()
        },
    );

    let pulse = engine.process_frame(start + Duration::from_millis(10), true, neutral());
    assert_eq!(pulse.buttons & BUTTON_Y, BUTTON_Y);
    assert_eq!(pulse.left_trigger, 255);

    let between = engine.process_frame(start + Duration::from_millis(100), true, neutral());
    assert_eq!(between.buttons & BUTTON_Y, 0);
    assert_eq!(between.left_trigger, 0);

    let second_pulse = engine.process_frame(start + Duration::from_millis(1010), true, neutral());
    assert_eq!(second_pulse.buttons & BUTTON_Y, BUTTON_Y);
    assert_eq!(second_pulse.left_trigger, 255);
}

#[test]
fn looping_keeps_other_buttons_but_lt_is_macro_owned() {
    let start = Instant::now();
    let mut engine = engine();

    engine.process_frame(
        start,
        true,
        PadState {
            buttons: BUTTON_Y,
            ..neutral()
        },
    );

    let output = engine.process_frame(
        start + Duration::from_millis(100),
        true,
        PadState {
            buttons: BUTTON_A,
            left_trigger: 200,
            ..neutral()
        },
    );

    assert_eq!(output.buttons & BUTTON_A, BUTTON_A);
    assert_eq!(output.left_trigger, 0);
}

#[test]
fn disconnect_returns_to_neutral_and_stops_looping() {
    let start = Instant::now();
    let mut engine = engine();

    engine.process_frame(
        start,
        true,
        PadState {
            buttons: BUTTON_Y,
            ..neutral()
        },
    );
    assert!(engine.is_looping());

    let output = engine.process_frame(start + Duration::from_millis(20), false, neutral());
    assert_eq!(output, PadState::neutral());
    assert!(!engine.is_looping());
}
