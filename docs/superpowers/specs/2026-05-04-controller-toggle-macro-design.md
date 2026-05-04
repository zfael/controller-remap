# Controller Toggle Macro Design

## Problem

Build a Windows-targeted Rust app that listens to a physical Xbox-like controller, uses the physical `Y` button as a toggle, and drives a virtual Xbox controller so the target app receives a repeating `Y + LT` pattern every 1 second while the macro is enabled. Pressing physical `Y` again stops the macro.

The physical `Y` press must be consumed so it does not reach the target app as a normal `Y` input.

## Proposed approach

Use a bridge/remapper architecture on Windows:

- Read the physical controller in the Rust app.
- Hide the physical controller from the target app.
- Expose a virtual Xbox controller to the target app.
- Use physical `Y` to toggle a two-state macro (`Idle` / `Looping`).
- When looping, emit `Y + LT` every 1 second through the virtual controller.

This approach is preferred because it is the only one that cleanly supports "physical `Y` toggles the macro but does not pass through" without duplicate or conflicting input.

## Architecture

The app sits between the physical controller and the target application:

`physical controller -> input reader -> macro state machine -> output mixer -> virtual controller -> target app`

### Units

1. **Physical input reader**
   - Reads the real controller on Windows.
   - Normalizes button transitions and held-state information.
2. **Macro state machine**
   - Owns the toggle state.
   - A rising edge on physical `Y` switches between `Idle` and `Looping`.
3. **Output mixer**
   - Produces the final virtual controller state for each tick.
   - Applies passthrough rules and macro injection rules.
4. **Virtual controller writer**
   - Publishes the computed state to the virtual Xbox controller that the target app sees.

## Runtime behavior

### Idle

- Physical `Y` is reserved for toggling only and is not forwarded.
- All other controls passthrough unchanged to the virtual controller.

### Looping

- Physical `Y` toggles the macro off and is not forwarded.
- The app emits a fixed `Y + LT` pattern every 1 second.
- Other non-`Y` inputs continue to passthrough.
- `LT` is owned by the macro while looping to avoid conflicts between the user's trigger state and the macro's trigger state.

### Disconnect handling

- If the physical controller disconnects, the macro stops immediately.
- The virtual controller is driven back to a neutral state so no button remains stuck.

## Platform assumptions

- Development and testing target is **Windows PC**.
- macOS is only the user's current planning environment.
- Standard Xbox-like buttons are the scope for the first version; vendor-specific Wolverine extra buttons are out of scope for this slice.
- The target app should see only the virtual controller path, not the original physical device path.

## Error handling

The app should fail fast with clear errors when:

- no supported physical controller is available,
- the virtual controller layer is unavailable,
- the app cannot acquire the physical device,
- or the required interception/hiding setup is missing.

The first version should prefer explicit failure over partial fallback behavior, because silent fallback could cause duplicate input or allow the physical `Y` toggle press to leak through.

## Success criteria

The proof of concept is successful when all of the following are true:

1. The app detects the physical `Y` press reliably.
2. Pressing `Y` once starts a repeating `Y + LT` loop at a 1 second cadence.
3. Pressing `Y` again stops the loop.
4. The target app sees the virtual loop behavior but does not see the physical `Y` toggle press.
5. Disconnecting the controller does not leave `Y` or `LT` stuck down.

## Non-goals for the first version

- Configurable timing
- Support for multiple macros
- UI/config editor
- Support for vendor-specific extra Wolverine buttons
- Cross-platform behavior beyond Windows

## Notes for implementation planning

- The key technical requirement is not just reading controller input; it is owning the full input path so the physical `Y` can be consumed.
- The implementation plan should explicitly choose the Windows controller-input API for reading the physical device and the virtual-controller mechanism for writing output.
- The implementation plan should include a small diagnostic path to prove the physical-to-virtual bridge before adding the toggle logic.
