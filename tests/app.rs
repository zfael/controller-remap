use std::time::{Duration, Instant};

use anyhow::Result;
use controller_remap::app::{BridgeApp, InputFrame, InputSource, OutputSink};
use controller_remap::domain::{PadState, BUTTON_A, BUTTON_Y};

#[derive(Default)]
struct FakeInput {
    frames: Vec<InputFrame>,
}

impl InputSource for FakeInput {
    fn poll(&mut self) -> Result<InputFrame> {
        Ok(self.frames.remove(0))
    }
}

#[derive(Default)]
struct FakeOutput {
    writes: Vec<PadState>,
}

impl OutputSink for FakeOutput {
    fn write(&mut self, state: PadState) -> Result<()> {
        self.writes.push(state);
        Ok(())
    }
}

#[test]
fn step_writes_the_macro_output_for_the_latest_frame() {
    let mut app = BridgeApp::new(
        FakeInput {
            frames: vec![InputFrame {
                connected: true,
                state: PadState {
                    buttons: BUTTON_Y | BUTTON_A,
                    ..PadState::neutral()
                },
            }],
        },
        FakeOutput::default(),
        Duration::from_secs(1),
        Duration::from_millis(60),
    );

    app.step(Instant::now()).unwrap();

    assert_eq!(app.output().writes.len(), 1);
    let write = app.output().writes[0];
    assert_eq!(write.buttons & BUTTON_A, BUTTON_A);
}

#[test]
fn disconnect_frame_writes_a_neutral_pad() {
    let mut app = BridgeApp::new(
        FakeInput {
            frames: vec![InputFrame {
                connected: false,
                state: PadState::neutral(),
            }],
        },
        FakeOutput::default(),
        Duration::from_secs(1),
        Duration::from_millis(60),
    );

    app.step(Instant::now()).unwrap();

    assert_eq!(app.output().writes, vec![PadState::neutral()]);
}
