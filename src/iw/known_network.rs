use anyhow::{Context, Result};
use iwdrs::known_netowk::KnownNetwork as IwdKnownNetwork;

#[derive(Debug, Clone)]
pub struct KnownNetwork {
    pub n: IwdKnownNetwork,
    pub name: String,
    pub network_type: String,
    pub is_autoconnect: bool,
    pub is_hidden: bool,
}

impl KnownNetwork {
    pub async fn new(n: IwdKnownNetwork) -> Result<Self> {
        let name = n.name().await?;
        let network_type = n.network_type().await?;

        let is_autoconnect = n
            .get_autoconnect()
            .await
            .context("Failed to check the autoconnect setting")?;

        let is_hidden = n.hidden().await?;

        Ok(Self {
            n,
            name,
            network_type,
            is_autoconnect,
            is_hidden,
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
