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

    loop {
        app.step(Instant::now())?;
        thread::sleep(Duration::from_millis(10));
    }
}
