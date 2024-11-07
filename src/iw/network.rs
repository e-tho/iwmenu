use crate::iw::known_network::KnownNetwork;
use anyhow::{anyhow, Result};
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
        let name = n.name().await?;
        let network_type = n.network_type().await?;
        let is_connected = n.connected().await?;
        let known_network = {
            match n.known_network().await {
                Ok(v) => match v {
                    Some(net) => Some(KnownNetwork::new(net).await.unwrap()),
                    None => None,
                },
                Err(_) => None,
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
