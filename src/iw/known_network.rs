use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use iwdrs::known_netowk::KnownNetwork as IwdKnownNetwork;
use tokio::sync::mpsc::UnboundedSender;

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

    pub async fn forget(&self, sender: UnboundedSender<String>) -> Result<()> {
        match self.n.forget().await {
            Ok(_) => {
                let msg = "Network Removed".to_string();
                sender
                    .send(msg)
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            }
            Err(e) => {
                let msg = e.to_string();
                sender
                    .send(msg)
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            }
        }
        Ok(())
    }

    pub async fn toggle_autoconnect(&self, sender: UnboundedSender<String>) -> Result<()> {
        if self.is_autoconnect {
            match self.n.set_autoconnect(false).await {
                Ok(_) => {
                    let msg = format!("Disable Autoconnect for: {}", self.name);
                    sender
                        .send(msg)
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                }
                Err(e) => {
                    let msg = e.to_string();
                    sender
                        .send(msg)
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                }
            }
        } else {
            match self.n.set_autoconnect(true).await {
                Ok(_) => {
                    let msg = format!("Enable Autoconnect for: {}", self.name);
                    sender
                        .send(msg)
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                }
                Err(e) => {
                    let msg = e.to_string();
                    sender
                        .send(msg)
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                }
            }
        }
        Ok(())
    }
}
