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
