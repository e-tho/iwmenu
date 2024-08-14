use anyhow::Result;
use iwdrs::netowrk::Network as IwdNetwork;
use notify_rust::Timeout;
use tokio::sync::mpsc::UnboundedSender;

use crate::iw::known_network::KnownNetwork;

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

    pub async fn connect(
        &self,
        sender: UnboundedSender<String>,
        notification_sender: UnboundedSender<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<Timeout>,
        )>,
    ) -> Result<()> {
        match self.n.connect().await {
            Ok(_) => {
                let msg = format!("Connected to {}", self.name);
                sender.send(msg.clone()).unwrap_or_else(|err| {
                    println!("Failed to send log message: {}", err);
                });
                notification_sender
                    .send((None, Some(msg.clone()), None, None))
                    .unwrap_or_else(|err| println!("Failed to send notification: {}", err));
            }
            Err(e) => {
                let msg = if e.to_string().contains("net.connman.iwd.Aborted") {
                    "Connection canceled".to_string()
                } else {
                    e.to_string()
                };
                sender.send(msg).unwrap_or_else(|err| {
                    println!("Failed to send log message: {}", err);
                });
            }
        }
        Ok(())
    }
}
