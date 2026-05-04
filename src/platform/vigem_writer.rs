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
        target
            .plugin()
            .context("failed to plug virtual controller")?;
        target
            .wait_ready()
            .context("virtual controller not ready")?;
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
