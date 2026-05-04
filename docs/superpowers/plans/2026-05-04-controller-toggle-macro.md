# Controller Toggle Macro Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows-only Rust app that reads a physical Xbox-style controller, uses physical `Y` as a toggle, and emits a virtual Xbox controller that pulses `Y + LT` once per second until `Y` is pressed again.

**Architecture:** Keep the core behavior in pure Rust domain code first: a macro engine that consumes physical `Y`, preserves normal passthrough in idle mode, and owns `LT` during looping. Then add thin Windows adapters: XInput for physical reads, ViGEmBus for virtual output, and a HidHide prerequisite check so the target app sees only the virtual controller path.

**Tech Stack:** Rust 2021, Cargo, `anyhow`, `clap`, `tracing-subscriber`, `rusty-xinput`, `vigem-client`, HidHide (external Windows prerequisite), ViGEmBus (external Windows prerequisite)

---

## File structure

- `Cargo.toml` — crate metadata and dependencies
- `.gitignore` — ignore build output and editor junk
- `README.md` — Windows setup notes for ViGEmBus, HidHide, and running the bridge
- `src/main.rs` — CLI entrypoint and tracing setup
- `src/lib.rs` — crate exports and `Cli` definition
- `src/app.rs` — app orchestration, trait boundaries, single-step bridge logic, Windows `run()`
- `src/domain/mod.rs` — domain exports
- `src/domain/pad_state.rs` — controller state model shared by input, macro, and output
- `src/domain/macro_engine.rs` — toggle behavior, 1-second cadence, 60 ms pulse width
- `src/platform/mod.rs` — platform module exports
- `src/platform/xinput_reader.rs` — physical controller reader using XInput
- `src/platform/vigem_writer.rs` — virtual Xbox writer using ViGEmBus
- `src/platform/prereqs.rs` — startup checks for required Windows prerequisites
- `tests/macro_engine.rs` — core macro behavior tests
- `tests/app.rs` — orchestration tests using fake input/output
- `tests/platform_conversions.rs` — pure conversion tests for XInput and ViGEm mapping

## Implementation notes locked in up front

- Use a **1 second interval** and a **60 ms pulse width** for the `Y + LT` press. This makes the meaning of "keep clicking" concrete for the proof of concept.
- Treat physical `Y` as **toggle-only**. It never reaches the virtual controller output.
- While looping, force `left_trigger = 255` during the pulse and `left_trigger = 0` between pulses. This satisfies the approved "macro owns LT while looping" rule.
- Keep all other buttons and sticks passing through unchanged unless they conflict with the macro-owned controls above.
- Fail fast on missing prerequisites instead of silently running in a broken "double input" state.

### Task 1: Bootstrap the Rust workspace

**Files:**
- Create: `.gitignore`
- Create: `Cargo.toml`
- Create: `README.md`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/app.rs`

- [ ] **Step 1: Initialize git and Cargo in the empty project directory**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
git init
cargo init --bin .
```

Expected:
- `Initialized empty Git repository`
- `Created binary (application) package`

- [ ] **Step 2: Replace the starter files with the minimal project skeleton**

Write `.gitignore`:

```gitignore
/target
/.idea
/.vscode
*.user
```

Write `Cargo.toml`:

```toml
[package]
name = "controller-remap"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

[target.'cfg(windows)'.dependencies]
rusty-xinput = "1.3.0"
vigem-client = "0.1.4"
```

Write `src/lib.rs`:

```rust
use anyhow::Result;
use clap::Parser;

pub mod app;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[arg(long, default_value_t = 0)]
    pub slot: u32,
}

pub fn run(cli: Cli) -> Result<()> {
    app::run(cli)
}
```

Write `src/main.rs`:

```rust
use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    controller_remap::run(controller_remap::Cli::parse())
}
```

Write `src/app.rs`:

```rust
use anyhow::Result;

use crate::Cli;

pub fn run(_cli: Cli) -> Result<()> {
    Ok(())
}
```

Write `README.md`:

```md
# controller-remap

Windows-only controller bridge proof of concept.

## Planned behavior

- Physical `Y` toggles the macro on and off.
- While enabled, the virtual controller pulses `Y + LT` every 1 second.
- The target app should see only the virtual controller.

## Prerequisites

- Windows PC
- ViGEmBus installed
- HidHide installed and configured to hide the physical controller from the target app while whitelisting this executable
```

- [ ] **Step 3: Run the bootstrap check**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo check
```

Expected: `Finished dev [unoptimized + debuginfo] target(s)` with no errors.

- [ ] **Step 4: Commit the bootstrap**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
git add .gitignore Cargo.toml README.md src/main.rs src/lib.rs src/app.rs
git commit -m "chore: bootstrap controller remap crate"
```

Expected: one commit containing only the bootstrap files above.

### Task 2: Implement the macro engine in pure domain code

**Files:**
- Create: `src/domain/mod.rs`
- Create: `src/domain/pad_state.rs`
- Create: `src/domain/macro_engine.rs`
- Modify: `src/lib.rs`
- Test: `tests/macro_engine.rs`

- [ ] **Step 1: Write the failing macro behavior tests**

Write `tests/macro_engine.rs`:

```rust
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
```

- [ ] **Step 2: Run the test file and verify it fails because the domain module does not exist yet**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo test --test macro_engine
```

Expected: FAIL with unresolved import errors for `controller_remap::domain`.

- [ ] **Step 3: Implement `PadState` and `MacroEngine` with the smallest code that satisfies the tests**

Write `src/domain/mod.rs`:

```rust
pub mod macro_engine;
pub mod pad_state;

pub use macro_engine::MacroEngine;
pub use pad_state::{PadState, BUTTON_A, BUTTON_Y};
```

Write `src/domain/pad_state.rs`:

```rust
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
```

Write `src/domain/macro_engine.rs`:

```rust
use std::time::{Duration, Instant};

use crate::domain::{PadState, BUTTON_Y};

#[derive(Debug, Clone)]
pub struct MacroEngine {
    looping: bool,
    y_was_down: bool,
    interval: Duration,
    pulse_width: Duration,
    next_pulse_at: Option<Instant>,
}

impl MacroEngine {
    pub fn new(interval: Duration, pulse_width: Duration) -> Self {
        Self {
            looping: false,
            y_was_down: false,
            interval,
            pulse_width,
            next_pulse_at: None,
        }
    }

    pub fn is_looping(&self) -> bool {
        self.looping
    }

    pub fn process_frame(&mut self, now: Instant, connected: bool, physical: PadState) -> PadState {
        if !connected {
            self.looping = false;
            self.y_was_down = false;
            self.next_pulse_at = None;
            return PadState::neutral();
        }

        let y_down = physical.buttons & BUTTON_Y != 0;
        if y_down && !self.y_was_down {
            self.looping = !self.looping;
            self.next_pulse_at = if self.looping { Some(now) } else { None };
        }
        self.y_was_down = y_down;

        let mut output = physical.without_buttons(BUTTON_Y);
        if !self.looping {
            return output;
        }

        output.left_trigger = 0;

        let next = self.next_pulse_at.get_or_insert(now);
        while now >= *next + self.interval {
            *next += self.interval;
        }

        if now >= *next && now < *next + self.pulse_width {
            output.buttons |= BUTTON_Y;
            output.left_trigger = 255;
        }

        output
    }
}
```

Modify `src/lib.rs`:

```rust
use anyhow::Result;
use clap::Parser;

pub mod app;
pub mod domain;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[arg(long, default_value_t = 0)]
    pub slot: u32,
}

pub fn run(cli: Cli) -> Result<()> {
    app::run(cli)
}
```

- [ ] **Step 4: Run the domain tests and the full test suite**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo test --test macro_engine
cargo test
```

Expected:
- `test physical_y_toggles_looping_only_on_rising_edge ... ok`
- `test idle_mode_passes_non_y_buttons_through_unchanged ... ok`
- `test looping_emits_y_and_lt_on_a_fixed_cadence ... ok`
- `test looping_keeps_other_buttons_but_lt_is_macro_owned ... ok`
- `test disconnect_returns_to_neutral_and_stops_looping ... ok`

- [ ] **Step 5: Commit the domain slice**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
git add src/lib.rs src/domain/mod.rs src/domain/pad_state.rs src/domain/macro_engine.rs tests/macro_engine.rs
git commit -m "feat: add macro engine domain logic"
```

Expected: one commit containing only the domain-model and tests files above.

### Task 3: Add an app boundary that can be tested without Windows devices

**Files:**
- Modify: `src/app.rs`
- Test: `tests/app.rs`

- [ ] **Step 1: Write the failing app orchestration tests with fakes**

Write `tests/app.rs`:

```rust
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
```

- [ ] **Step 2: Run the app tests and verify they fail because the traits and bridge type do not exist yet**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo test --test app
```

Expected: FAIL with unresolved imports for `BridgeApp`, `InputFrame`, `InputSource`, or `OutputSink`.

- [ ] **Step 3: Implement the testable bridge boundary in `src/app.rs`**

Replace `src/app.rs` with:

```rust
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
```

- [ ] **Step 4: Run the new app tests and then the whole suite**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo test --test app
cargo test
```

Expected:
- `test step_writes_the_macro_output_for_the_latest_frame ... ok`
- `test disconnect_frame_writes_a_neutral_pad ... ok`
- all previous `macro_engine` tests still pass

- [ ] **Step 5: Commit the bridge boundary**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
git add src/app.rs tests/app.rs
git commit -m "feat: add testable bridge app boundary"
```

Expected: one commit containing the app boundary and its tests.

### Task 4: Add the Windows adapters, prerequisite checks, and the real run loop

**Files:**
- Create: `src/platform/mod.rs`
- Create: `src/platform/xinput_reader.rs`
- Create: `src/platform/vigem_writer.rs`
- Create: `src/platform/prereqs.rs`
- Modify: `src/app.rs`
- Modify: `src/lib.rs`
- Modify: `README.md`
- Test: `tests/platform_conversions.rs`

- [ ] **Step 1: Write the failing conversion tests for the Windows adapter layer**

Write `tests/platform_conversions.rs`:

```rust
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
```

- [ ] **Step 2: Run the conversion tests and verify they fail because the platform module does not exist yet**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo test --test platform_conversions
```

Expected: FAIL with unresolved imports for `controller_remap::platform`.

- [ ] **Step 3: Implement the Windows adapters and wire `run()` to the real event loop**

Write `src/platform/mod.rs`:

```rust
pub mod prereqs;
pub mod vigem_writer;
pub mod xinput_reader;
```

Write `src/platform/xinput_reader.rs`:

```rust
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
```

Write `src/platform/vigem_writer.rs`:

```rust
use anyhow::{Context, Result};
use vigem_client::{Client, TargetId, XGamepad, Xbox360Wired};

use crate::app::OutputSink;
use crate::domain::PadState;

pub struct VigemWriter {
    target: Xbox360Wired<Client>,
}

impl VigemWriter {
    pub fn new() -> Result<Self> {
        let client = Client::connect().context("failed to connect to ViGEmBus")?;
        let mut target = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
        target.plugin().context("failed to plug virtual controller")?;
        target.wait_ready().context("virtual controller not ready")?;
        Ok(Self { target })
    }
}

pub fn to_xgamepad(state: PadState) -> XGamepad {
    XGamepad {
        buttons: state.buttons.into(),
        left_trigger: state.left_trigger,
        right_trigger: state.right_trigger,
        thumb_lx: state.thumb_lx,
        thumb_ly: state.thumb_ly,
        thumb_rx: state.thumb_rx,
        thumb_ry: state.thumb_ry,
    }
}

impl OutputSink for VigemWriter {
    fn write(&mut self, state: PadState) -> Result<()> {
        self.target
            .update(&to_xgamepad(state))
            .context("failed to update virtual controller")?;
        Ok(())
    }
}
```

Write `src/platform/prereqs.rs`:

```rust
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn check_prerequisites() -> Result<()> {
    let output = Command::new("sc")
        .args(["query", "HidHide"])
        .output()
        .context("failed to query HidHide service")?;

    if !output.status.success() {
        bail!("HidHide service not found; install HidHide and whitelist this executable before running");
    }

    Ok(())
}
```

Modify `src/app.rs`:

```rust
use std::thread;
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
```

Modify `src/lib.rs`:

```rust
use anyhow::Result;
use clap::Parser;

pub mod app;
pub mod domain;
#[cfg(windows)]
pub mod platform;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[arg(long, default_value_t = 0)]
    pub slot: u32,
}

pub fn run(cli: Cli) -> Result<()> {
    app::run(cli)
}
```

Modify `README.md`:

```md
# controller-remap

Windows-only controller bridge proof of concept.

## Behavior

- Physical `Y` toggles the macro on and off.
- While enabled, the virtual controller pulses `Y + LT` every 1 second.
- Pulse width is 60 ms.
- Physical `Y` does not pass through to the target app.

## Windows prerequisites

1. Install ViGEmBus.
2. Install HidHide.
3. Add the built executable to the HidHide application allow-list.
4. Hide the physical controller from the target app while allowing this bridge executable to see it.

## Run

```bash
cargo run -- --slot 0
```

If HidHide is missing, the app should exit with a clear startup error instead of running in a broken state.
```

- [ ] **Step 4: Run tests, then do the Windows-only manual smoke check**

Run automated tests:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo test --test platform_conversions
cargo test
```

Expected:
- `test pad_state_maps_to_vigem_gamepad ... ok`
- `test xinput_state_maps_to_pad_state ... ok`
- all earlier tests still pass

Then on Windows with prerequisites installed:

```bash
cd /Users/homelab/dev/apps/controller-remap
cargo run -- --slot 0
```

Manual verification checklist:
- The app starts without prerequisite errors.
- Windows plays the virtual-controller connect sound.
- Pressing physical `Y` starts the macro.
- The target app receives `Y + LT` every 1 second.
- Pressing physical `Y` again stops the macro.
- Disconnecting the controller leaves no stuck inputs.

- [ ] **Step 5: Commit the Windows integration slice**

Run:

```bash
cd /Users/homelab/dev/apps/controller-remap
git add README.md src/lib.rs src/app.rs src/platform/mod.rs src/platform/xinput_reader.rs src/platform/vigem_writer.rs src/platform/prereqs.rs tests/platform_conversions.rs
git commit -m "feat: wire windows controller bridge"
```

Expected: one commit containing the platform adapters, real run loop, README update, and conversion tests.
