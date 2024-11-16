use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use iwdrs::known_netowk::KnownNetwork as IwdKnownNetwork;

#[derive(Debug, Clone)]
pub struct KnownNetwork {
    pub n: IwdKnownNetwork,
    pub name: String,
    pub network_type: String,
    pub is_autoconnect: bool,
    pub is_hidden: bool,
    pub last_connected: Option<DateTime<FixedOffset>>,
}

impl KnownNetwork {
    pub async fn new(n: IwdKnownNetwork) -> Result<Self> {
        let name = n
            .name()
            .await
            .context("Failed to retrieve the known network name")?;

        let network_type = n
            .network_type()
            .await
            .context("Failed to retrieve the known network type")?;

        let is_autoconnect = n
            .get_autoconnect()
            .await
            .context("Failed to check the autoconnect setting")?;

        let is_hidden = n
            .hidden()
            .await
            .context("Failed to check if the known network is hidden")?;

        let last_connected = n
            .last_connected_time()
            .await
            .ok()
            .and_then(|v| DateTime::parse_from_rfc3339(&v).ok());

        Ok(Self {
            n,
            name,
            network_type,
            is_autoconnect,
            is_hidden,
            last_connected,
        })
    }

    pub async fn forget(&self) -> Result<()> {
        self.n
            .forget()
            .await
            .context("Failed to forget the known network")
    }

    pub async fn toggle_autoconnect(&self, enable: bool) -> Result<()> {
        self.n
            .set_autoconnect(enable)
            .await
            .context("Failed to toggle the autoconnect setting")
    }
}
