# controller-remap

Windows-only controller bridge proof of concept.

## Behavior

- Physical `Y` toggles the macro on and off.
- While enabled, the virtual controller pulses `Y + LT` every 1 second.
- Pulse width is 60 ms.
- Physical `Y` does not pass through to the target app.
- Physical `LT` does not pass through while the macro is enabled; the macro owns `LT` timing.

## Windows setup

This bridge currently depends on two Windows components:

- **ViGEmBus** creates the virtual Xbox 360 controller that the target app sees. It is **archived/retired upstream**, but the current implementation still depends on it. For this proof of concept, treat it as a legacy prerequisite rather than a long-term dependency choice. See the official end-of-life note: <https://docs.nefarius.at/projects/ViGEm/End-of-Life/>.
- **HidHide** hides the physical controller from ordinary applications while still allowing this bridge executable to read it.

You will need:

- Windows 10 or 11
- administrator rights to install drivers

### 1. Install Rust and Cargo

1. Install Rust from <https://rustup.rs/>.
2. Use the default installation options unless you already have a reason to customize them.
3. Close and reopen your terminal after installation so `cargo` is available on `PATH`.
4. Confirm the toolchain is available:

```bash
cargo --version
```

If `cargo` is not recognized, reopen the terminal and try again before continuing.

### 2. Build the executable

```bash
cargo build --release
```

The executable will be at `target\release\controller-remap.exe`. You will need that exact file path for the HidHide allow-list step.

### 3. Install ViGEmBus

1. Download the latest official archived release from <https://github.com/nefarius/ViGEmBus/releases/latest>.
2. Run the installer.
3. Accept any Windows driver prompts.
4. Reboot if the installer asks for it.

The bridge creates a virtual Xbox 360 controller through ViGEmBus. If it is missing, startup will fail with `failed to connect to ViGEmBus`.

### 4. Install HidHide

1. Download the latest official release from <https://github.com/nefarius/HidHide/releases/latest>.
2. Run the installer.
3. Reboot if prompted.
4. Open **HidHide Configuration Client**.

> HidHide controls which applications are allowed to see hidden devices. This bridge must be allow-listed before device hiding is enabled.

### 5. Allow this executable through HidHide

1. Open the **Applications** tab.
2. Press **+**.
3. Browse to `target\release\controller-remap.exe`.
4. Add it to the list.

If the bridge executable is not allow-listed, HidHide can hide the physical controller from the bridge too.

### 6. Hide the physical controller from the target app

1. Before continuing, make sure the physical controller is connected.
2. Open the **Devices** tab.
3. Find the physical controller in the device list.
4. If it does not appear, disable **Gaming devices only** and check again.
5. Select the physical controller for hiding.
6. Turn on **Enable device hiding**.

After this:

- `controller-remap.exe` can still see the physical controller because it is allow-listed.
- The target app should no longer see the physical controller directly.
- The target app should interact with the virtual controller created by ViGEmBus instead.

### 7. Run the bridge

```bash
cargo run --release -- --slot 0
```

`--slot` is the XInput slot index to read from. If the controller is not detected, confirm which slot Windows assigned to the physical controller.

When ViGEmBus creates the virtual Xbox 360 controller, Windows can assign that virtual device to slot `0` and move the physical controller to slot `1`. If pressing physical `Y` does not produce `macro enabled` in the bridge logs, try the next slot:

```bash
cargo run --release -- --slot 1
```

### 8. Verify the setup

1. Start the bridge with the physical controller connected.
2. Confirm startup does not fail with a HidHide error.
3. Confirm the bridge logs `physical controller connected` and, when you press physical `Y`, logs `macro enabled`.
4. If physical `Y` produces no new bridge log, stop and retry with a different `--slot` value before opening the target app.
5. Open the target app and confirm it reacts to the virtual controller path instead of seeing both physical and virtual devices at the same time.
6. Press physical `Y` and confirm it acts as the bridge toggle instead of appearing as a normal `Y` press in the target app.

### 9. Common setup failures

- `cargo` is not recognized
  - Rust is not installed yet, or the terminal needs to be reopened after installing via `rustup`.
- `HidHide service not found; install HidHide and whitelist this executable before running`
  - HidHide is not installed correctly, its service is unavailable, or the bridge was started before installation completed.
- `failed to connect to ViGEmBus`
  - ViGEmBus is missing, not active yet, or Windows still needs a reboot after installation.
- The bridge logs `physical controller connected`, but pressing physical `Y` does nothing
  - The selected `--slot` is likely the virtual Xbox 360 controller instead of the physical pad. Try `--slot 1` if `--slot 0` stays idle.
- The controller is missing from the HidHide **Devices** tab
  - Reconnect the controller and disable **Gaming devices only**.
- The bridge stops seeing the controller after hiding is enabled
  - Re-check that `controller-remap.exe` is listed on the HidHide **Applications** tab.

> Official HidHide documentation notes that Kaspersky Anti-Virus can interfere with application whitelisting.

## Run

```bash
cargo run --release -- --slot 0
```

If HidHide is missing, the app should exit with a clear startup error instead of running in a broken state.
