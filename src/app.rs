use std::time::{Duration, Instant};

#[cfg(not(windows))]
use anyhow::bail;
use anyhow::Result;
use tracing::info;

use crate::domain::{MacroEngine, PadState, BUTTON_Y};
#[cfg(windows)]
use crate::platform::prereqs::check_prerequisites;
#[cfg(windows)]
use crate::platform::vigem_writer::VigemWriter;
#[cfg(windows)]
use crate::platform::xinput_reader::XInputReader;
use crate::Cli;
#[cfg(windows)]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
#[cfg(windows)]
use std::thread;

#[derive(Debug, Clone, Copy)]
pub struct InputFrame {
    pub connected: bool,
    pub state: PadState,
}

pub trait InputSource {
    fn poll(&mut self) -> Result<InputFrame>;
}

pub trait OutputSink {
    fn write(&mut self, state: PadState) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeEvent {
    PhysicalControllerConnected,
    PhysicalControllerDisconnected,
    MacroEnabled,
    MacroDisabled,
    MacroPulse,
}

pub struct BridgeApp<I, O> {
    input: I,
    output: O,
    engine: MacroEngine,
    last_connected: Option<bool>,
    last_pulse_active: bool,
}

impl<I, O> BridgeApp<I, O>
where
    I: InputSource,
    O: OutputSink,
{
    pub fn new(input: I, output: O, interval: Duration, pulse_width: Duration) -> Self {
        Self {
            input,
            output,
            engine: MacroEngine::new(interval, pulse_width),
            last_connected: None,
            last_pulse_active: false,
        }
    }

    pub fn step(&mut self, now: Instant) -> Result<()> {
        self.step_with_events(now).map(|_| ())
    }

    pub fn step_with_events(&mut self, now: Instant) -> Result<Vec<BridgeEvent>> {
        let frame = self.input.poll()?;
        let was_looping = self.engine.is_looping();
        let next = self.engine.process_frame(now, frame.connected, frame.state);
        let is_looping = self.engine.is_looping();
        let mut events = Vec::new();

        if self.last_connected != Some(frame.connected) {
            events.push(if frame.connected {
                BridgeEvent::PhysicalControllerConnected
            } else {
                BridgeEvent::PhysicalControllerDisconnected
            });
            self.last_connected = Some(frame.connected);
        }

        if was_looping != is_looping {
            events.push(if is_looping {
                BridgeEvent::MacroEnabled
            } else {
                BridgeEvent::MacroDisabled
            });
        }

        let pulse_active = is_looping && next.buttons & BUTTON_Y != 0 && next.left_trigger == 255;
        if pulse_active && !self.last_pulse_active {
            events.push(BridgeEvent::MacroPulse);
        }
        self.last_pulse_active = pulse_active;

        self.output.write(next)?;
        Ok(events)
    }

    pub fn output(&self) -> &O {
        &self.output
    }
}

#[cfg(any(windows, test))]
fn run_until_stopped<I, O, C, S, E>(
    app: &mut BridgeApp<I, O>,
    mut should_continue: C,
    mut sleep: S,
    mut on_event: E,
) -> Result<()>
where
    I: InputSource,
    O: OutputSink,
    C: FnMut() -> bool,
    S: FnMut(Duration),
    E: FnMut(BridgeEvent),
{
    while should_continue() {
        for event in app.step_with_events(Instant::now())? {
            on_event(event);
        }
        sleep(Duration::from_millis(10));
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn run(_cli: Cli) -> Result<()> {
    bail!("controller-remap currently supports only Windows")
}

#[cfg(windows)]
pub fn run(cli: Cli) -> Result<()> {
    check_prerequisites()?;

    let input = XInputReader::new(cli.slot)?;
    let output = VigemWriter::new()?;
    let mut app = BridgeApp::new(
        input,
        output,
        Duration::from_secs(1),
        Duration::from_millis(60),
    );
    let running = Arc::new(AtomicBool::new(true));
    let signal = Arc::clone(&running);

    ctrlc::set_handler(move || {
        signal.store(false, Ordering::SeqCst);
    })?;

    info!(slot = cli.slot, "bridge started");

    run_until_stopped(
        &mut app,
        || running.load(Ordering::SeqCst),
        thread::sleep,
        |event| log_bridge_event(cli.slot, event),
    )
}

#[cfg(windows)]
fn log_bridge_event(slot: u32, event: BridgeEvent) {
    match event {
        BridgeEvent::PhysicalControllerConnected => {
            info!(slot = slot, "physical controller connected")
        }
        BridgeEvent::PhysicalControllerDisconnected => {
            info!(slot = slot, "physical controller disconnected")
        }
        BridgeEvent::MacroEnabled => info!(slot = slot, "macro enabled"),
        BridgeEvent::MacroDisabled => info!(slot = slot, "macro disabled"),
        BridgeEvent::MacroPulse => info!(slot = slot, "macro pulse fired"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn run_until_stopped_exits_cleanly_when_shutdown_is_requested() {
        let mut app = BridgeApp::new(
            FakeInput {
                frames: vec![
                    InputFrame {
                        connected: true,
                        state: PadState::neutral(),
                    },
                    InputFrame {
                        connected: true,
                        state: PadState::neutral(),
                    },
                ],
            },
            FakeOutput::default(),
            Duration::from_secs(1),
            Duration::from_millis(60),
        );

        let mut iterations = 0;
        run_until_stopped(
            &mut app,
            || {
                let keep_running = iterations < 2;
                iterations += 1;
                keep_running
            },
            |_| {},
            |_| {},
        )
        .unwrap();

        assert_eq!(app.output().writes, vec![PadState::neutral(), PadState::neutral()]);
    }
}
