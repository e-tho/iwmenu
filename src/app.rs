use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    iw::{agent::AgentManager, station::Station},
    menu::Menu,
};

pub struct App {
    station: Station,
    agent_manager: AgentManager,
    log_sender: UnboundedSender<String>,
}

impl App {
    pub async fn new(_menu: Menu, log_sender: UnboundedSender<String>) -> Result<Self> {
        let agent_manager = AgentManager::new().await?;
        let session = agent_manager.session();

        let station = Station::new(session.clone()).await?;

        Ok(Self {
            station,
            agent_manager,
            log_sender,
        })
    }

    pub async fn run(&self, menu: Menu) -> Result<()> {
        if let Some(ssid) = menu.select_ssid(&self.station).await? {
            let (network, _) = self
                .station
                .new_networks
                .iter()
                .chain(self.station.known_networks.iter())
                .find(|(network, _)| network.name == ssid)
                .unwrap();

            if let Some(known_network) = &network.known_network {
                if known_network.is_autoconnect {
                    self.log_sender
                        .send(format!(
                            "Auto-connecting to known network: {}",
                            network.name
                        ))
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    network.connect(self.log_sender.clone()).await?;
                    return Ok(());
                }
            }

            if let Some(passphrase) = menu.prompt_passphrase(&network.name) {
                self.agent_manager.send_passkey(passphrase)?;
            } else {
                self.agent_manager.cancel_auth()?;
            }

            network.connect(self.log_sender.clone()).await?;
        } else {
            self.log_sender
                .send("No network selected".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
        }

        Ok(())
    }
}
