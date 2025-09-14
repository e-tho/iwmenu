use crate::iw::device::Device;
use anyhow::{anyhow, Context, Result};
use iwdrs::{adapter::Adapter as IwdAdapter, session::Session};
use log::warn;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Adapter {
    pub adapter: IwdAdapter,
    pub is_powered: bool,
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub supported_modes: Vec<String>,
    pub device: Device,
}

impl Adapter {
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let adapter = session
            .adapter()
            .ok_or_else(|| anyhow!("No adapter found"))?;

        let is_powered = adapter
            .is_powered()
            .await
            .context("Failed to get adapter power state")?;

        let name = adapter.name().await?;

        let model = adapter
            .model()
            .await
            .map_err(|e| {
                warn!("Failed to get adapter model: {e}");
            })
            .ok();

        let vendor = adapter
            .vendor()
            .await
            .map_err(|e| {
                warn!("Failed to get adapter vendor: {e}");
            })
            .ok();

        let supported_modes = adapter.supported_modes().await?;

        let device = Device::new(session.clone())
            .await
            .context("Failed to initialize device")?;

        Ok(Self {
            adapter,
            is_powered,
            name,
            model,
            vendor,
            supported_modes,
            device,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.is_powered = self.adapter.is_powered().await?;

        self.device
            .refresh()
            .await
            .context("Failed to refresh device")?;

        Ok(())
    }
}
