use anyhow::Result;
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
        let name = n.name().await?;
        let network_type = n.network_type().await?;
        let is_autoconnect = n.get_autoconnect().await?;
        let is_hidden = n.hidden().await?;
        let last_connected = match n.last_connected_time().await {
            Ok(v) => DateTime::parse_from_rfc3339(&v).ok(),
            Err(_) => None,
        };

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
        self.n.forget().await?;
        Ok(())
    }

    pub async fn toggle_autoconnect(&self, enable: bool) -> Result<()> {
        self.n.set_autoconnect(enable).await?;
        Ok(())
    }
}
