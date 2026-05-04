# Windows Setup README Enrichment Design

## Problem

The current `README.md` only lists the Windows prerequisites at a high level:

1. Install ViGEmBus.
2. Install HidHide.
3. Add the built executable to the HidHide application allow-list.
4. Hide the physical controller from the target app while allowing this bridge executable to see it.

That is not detailed enough for a tester sitting at a Windows machine to complete setup without guessing which installers to use, which HidHide tabs to open, what to select, or how to tell whether setup is correct.

## Goal

Rewrite the Windows prerequisites portion of the README into a practical setup guide that a Windows tester can follow end-to-end to:

- install the required drivers and tools,
- understand which prerequisite is legacy/deprecated versus actively maintained,
- build or locate the bridge executable,
- configure HidHide so the bridge can still see the physical controller,
- hide the physical controller from the target app,
- and verify that the environment is ready before testing the macro behavior.

## Non-goals

- Automating installation of ViGEmBus or HidHide from this project.
- Explaining every HidHide feature beyond what this bridge needs.
- Supporting non-Windows platforms.
- Documenting advanced driver debugging or ARM64-specific deployment.

## Approaches considered

### 1. Expand the README with an inline Windows setup guide (**recommended**)

Add a new README section that walks through the full setup in order:

- what to install and where to get it,
- how to build the executable,
- how to add the executable to HidHide's Applications list,
- how to select the physical controller in HidHide's Devices list,
- how to enable device hiding,
- how to run the bridge and confirm the setup worked,
- and what the most likely setup failures look like.

This is the best fit because the setup is short enough to keep in the README, and the user specifically wants something they can follow directly on a Windows machine without jumping between multiple project docs.

### 2. Keep the README short and link to a dedicated Windows setup document

This would reduce README length and allow deeper explanations later, but it adds one more navigation step for the very first thing a tester needs to do.

This is reasonable if the setup becomes much longer in the future, but it is not the best choice for the current project size.

### 3. Add only a troubleshooting appendix and leave the main bullets short

This would help after setup fails, but it would still leave the main path underspecified.

This does not solve the primary problem: a new tester still would not know the correct sequence of actions.

## Recommended design

The README should keep the existing high-level structure, but replace the current four-bullet prerequisite section with a procedural Windows setup guide that can be followed top to bottom.

### Section structure

The enriched documentation should be organized like this:

1. **Windows setup**
2. **Build the executable**
3. **Install ViGEmBus**
4. **Install HidHide**
5. **Allow this executable through HidHide**
6. **Hide the physical controller from the target app**
7. **Run the bridge**
8. **Verify the setup**
9. **Common setup failures**

The heading names can be adjusted for readability, but the sequence should stay procedural.

## Content design

### 1. Windows setup overview

Open with a short explanation of why both tools are needed:

- **ViGEmBus** is needed because the app writes to a virtual Xbox 360 controller.
- **HidHide** is needed because the target app must not see the physical controller at the same time as the virtual one.

Also state the practical requirement that the tester needs Windows 10 or newer, admin rights to install drivers, and a connected controller before the HidHide device-selection step.

### 2. Build the executable

Document the Windows-native build path explicitly:

```bash
cargo build --release
```

Then point to the expected output path:

`target\release\controller-remap.exe`

This matters because the HidHide allow-list step needs the actual executable path, not just the project directory.

### 3. Install ViGEmBus

Document the user-facing installation flow:

- Explicitly call out that **ViGEmBus is archived/retired upstream** but is still required by the current implementation of this project.
- Download the latest official archived ViGEmBus release from the official Nefarius GitHub/releases source.
- Run the installer.
- Accept any driver-install prompts from Windows.
- Reboot if the installer asks for it.

The README should avoid over-explaining ViGEm internals, but it should say what success means: the bridge should later be able to create a virtual Xbox 360 controller, and if ViGEmBus is missing the app will fail when it tries to connect to the virtual bus.

The README should also add one sentence of risk framing: this is acceptable for local proof-of-concept testing because the official end-of-life statement says existing installations continue to work, but it should be treated as a legacy dependency rather than an actively maintained foundation.

### 4. Install HidHide

Document the simplest supported path:

- Download the latest official HidHide release from the official Nefarius GitHub/releases source.
- Run the installer.
- Reboot if prompted.
- Open the HidHide Configuration Client after installation.

The README should mention that HidHide controls which applications can still see hidden devices, because that concept explains the next two steps.

### 5. Allow this executable through HidHide

This should be a concrete GUI walkthrough:

- Open the **Applications** tab in the HidHide Configuration Client.
- Press **+**.
- Browse to `target\release\controller-remap.exe` (or wherever the built executable was copied).
- Add it to the list.

The README should explain the purpose plainly: if the bridge executable is not allow-listed, HidHide can block the bridge from seeing the physical controller too, which breaks the remap path.

### 6. Hide the physical controller from the target app

This is the most important part and needs the clearest detail:

- Connect the physical controller before opening or refreshing the **Devices** tab.
- Open the **Devices** tab in HidHide.
- Find the physical controller in the device list.
- If the controller is not visible, disable the **Gaming devices only** filter and check again.
- Select the physical controller for hiding.
- Turn on **Enable device hiding**.

The README should explicitly explain the intended end state:

- the bridge executable can still see the physical controller because it is allow-listed,
- ordinary applications, including the target app, should no longer see that physical controller,
- the target app should instead interact with the virtual controller created by ViGEmBus.

### 7. Run the bridge

Keep the current run command, but tie it back to the earlier setup:

```bash
cargo run --release -- --slot 0
```

Also clarify that `--slot` is the XInput slot index to read from, so if the controller is not being detected the tester may need to confirm which slot the physical controller occupies.

### 8. Verify the setup

Add a short checklist the tester can perform before opening the target app:

1. Start the bridge with the physical controller connected.
2. Confirm the app does not immediately fail with a HidHide error.
3. Open the target app and confirm it reacts to the virtual controller path rather than seeing both physical and virtual devices simultaneously.
4. Press physical `Y` and confirm it behaves as a toggle for the bridge behavior instead of appearing as a normal `Y` press in the target app.

This section should be written as practical checks, not abstract success criteria.

### 9. Common setup failures

Keep this brief and directly tied to the current implementation:

- **`HidHide service not found; install HidHide and whitelist this executable before running`**
  - HidHide is not installed correctly, its service is unavailable, or setup was attempted before installation completed.
- **`failed to connect to ViGEmBus` / virtual-controller startup errors**
  - ViGEmBus is not installed, not active yet, or Windows still needs a reboot after driver installation.
- **Controller is missing from HidHide's device list**
  - Reconnect it and disable the **Gaming devices only** filter.
- **Bridge cannot see the controller after hiding is enabled**
  - Re-check that `controller-remap.exe` is present in the HidHide Applications allow-list.

The README should also include a short note from the official HidHide docs that third-party antivirus products, specifically Kaspersky, can interfere with application whitelisting.

## Tone and formatting

The updated README should favor:

- numbered steps for the primary setup path,
- short explanatory notes only where they prevent common mistakes,
- exact executable names and UI tab names,
- one clear caveat where the project depends on archived ViGEmBus,
- and verification bullets immediately after setup, not far away in a separate appendix.

The writing should stay practical and test-oriented, not tutorial-heavy.

## Success criteria

The README enrichment is successful when a Windows tester can use only the project README plus the linked official ViGEmBus/HidHide release pages to:

1. install the required drivers and utilities,
2. build or locate the correct executable,
3. configure HidHide correctly,
4. run the bridge without hidden-device misconfiguration,
5. and validate that the target app no longer sees the physical toggle press directly.

## Notes for implementation

- Prefer official Nefarius download/release sources in the README instead of third-party mirrors.
- Make it explicit that ViGEmBus is a legacy prerequisite of the current codebase, not a recommendation for new long-term platform investment.
- Keep the current concise behavior section intact unless the setup flow needs a tiny cross-reference.
- Use the app's real error strings where helpful so the README matches what the tester will actually see.
