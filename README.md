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
