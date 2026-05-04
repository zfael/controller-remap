use std::time::{Duration, Instant};

use anyhow::Result;

use crate::domain::{MacroEngine, PadState};
use crate::Cli;

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

pub fn run(_cli: Cli) -> Result<()> {
    Ok(())
}
