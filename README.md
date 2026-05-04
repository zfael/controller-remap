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

### 1. Build the executable

```bash
cargo build --release
```

The executable will be at `target\release\controller-remap.exe`. You will need that exact file path for the HidHide allow-list step.

### 2. Install ViGEmBus

1. Download the latest official archived release from <https://github.com/nefarius/ViGEmBus/releases/latest>.
2. Run the installer.
3. Accept any Windows driver prompts.
4. Reboot if the installer asks for it.

The bridge creates a virtual Xbox 360 controller through ViGEmBus. If it is missing, startup will fail with `failed to connect to ViGEmBus`.

### 3. Install HidHide

1. Download the latest official release from <https://github.com/nefarius/HidHide/releases/latest>.
2. Run the installer.
3. Reboot if prompted.
4. Open **HidHide Configuration Client**.

> HidHide controls which applications are allowed to see hidden devices. This bridge must be allow-listed before device hiding is enabled.

### 4. Allow this executable through HidHide

1. Open the **Applications** tab.
2. Press **+**.
3. Browse to `target\release\controller-remap.exe`.
4. Add it to the list.

If the bridge executable is not allow-listed, HidHide can hide the physical controller from the bridge too.

### 5. Hide the physical controller from the target app

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

### 6. Run the bridge

```bash
cargo run --release -- --slot 0
```

`--slot` is the XInput slot index to read from. If the controller is not detected, confirm which slot Windows assigned to the controller.

### 7. Verify the setup

1. Start the bridge with the physical controller connected.
2. Confirm startup does not fail with a HidHide error.
3. Open the target app and confirm it reacts to the virtual controller path instead of seeing both physical and virtual devices at the same time.
4. Press physical `Y` and confirm it acts as the bridge toggle instead of appearing as a normal `Y` press in the target app.

### 8. Common setup failures

- `HidHide service not found; install HidHide and whitelist this executable before running`
  - HidHide is not installed correctly, its service is unavailable, or the bridge was started before installation completed.
- `failed to connect to ViGEmBus`
  - ViGEmBus is missing, not active yet, or Windows still needs a reboot after installation.
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
