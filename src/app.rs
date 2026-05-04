use std::time::{Duration, Instant};

use anyhow::{bail, Result};

use crate::domain::{MacroEngine, PadState};
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

pub struct BridgeApp<I, O> {
    input: I,
    output: O,
    engine: MacroEngine,
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
        }
    }

    pub fn step(&mut self, now: Instant) -> Result<()> {
        let frame = self.input.poll()?;
        let next = self.engine.process_frame(now, frame.connected, frame.state);
        self.output.write(next)
    }

    pub fn output(&self) -> &O {
        &self.output
    }
}

#[cfg(any(windows, test))]
fn run_until_stopped<I, O, C, S>(
    app: &mut BridgeApp<I, O>,
    mut should_continue: C,
    mut sleep: S,
) -> Result<()>
where
    I: InputSource,
    O: OutputSink,
    C: FnMut() -> bool,
    S: FnMut(Duration),
{
    while should_continue() {
        app.step(Instant::now())?;
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

    run_until_stopped(
        &mut app,
        || running.load(Ordering::SeqCst),
        thread::sleep,
    )
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
        )
        .unwrap();

        assert_eq!(app.output().writes, vec![PadState::neutral(), PadState::neutral()]);
    }
}
