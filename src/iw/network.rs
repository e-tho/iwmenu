use crate::iw::known_network::KnownNetwork;
use anyhow::{anyhow, Context, Result};
use iwdrs::netowrk::Network as IwdNetwork;

#[derive(Debug, Clone)]
pub struct Network {
    pub n: IwdNetwork,
    pub name: String,
    pub network_type: String,
    pub is_connected: bool,
    pub known_network: Option<KnownNetwork>,
}

impl Network {
    pub async fn new(n: IwdNetwork) -> Result<Self> {
        let name = n
            .name()
            .await
            .context("Failed to retrieve the network name")?;

        let network_type = n
            .network_type()
            .await
            .context("Failed to retrieve the network type")?;

        let is_connected = n
            .connected()
            .await
            .context("Failed to check if the network is connected")?;

        let known_network = match n.known_network().await {
            Ok(Some(net)) => Some(
                KnownNetwork::new(net)
                    .await
                    .context("Failed to initialize the known network")?,
            ),
            Ok(None) => None,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to retrieve known network information: {}",
                    e
                );
                None
            }
        };

        Ok(Self {
            n,
            name,
            network_type,
            is_connected,
            known_network,
        })
    }

    pub async fn connect(&self) -> Result<()> {
        self.n.connect().await.map_err(|e| {
            if e.to_string().contains("net.connman.iwd.Aborted") {
                anyhow!(t!("notifications.network.connection_canceled"))
            } else {
                e
            }
        })
    }
}
