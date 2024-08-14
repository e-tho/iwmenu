use anyhow::Result;
use notify_rust::Timeout;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    iw::{agent::AgentManager, station::Station},
    menu::Menu,
};

pub struct App {
    station: Station,
    agent_manager: AgentManager,
    log_sender: UnboundedSender<String>,
    notification_sender: UnboundedSender<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<Timeout>,
    )>,
}

impl App {
    pub async fn new(
        _menu: Menu,
        log_sender: UnboundedSender<String>,
        notification_sender: UnboundedSender<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<Timeout>,
        )>,
    ) -> Result<Self> {
        let agent_manager = AgentManager::new().await?;
        let session = agent_manager.session();

        let station = Station::new(session.clone()).await?;

        Ok(Self {
            station,
            agent_manager,
            log_sender,
            notification_sender,
        })
    }

    pub async fn run(&mut self, menu: Menu, icon_type: &str) -> Result<Option<String>> {
        loop {
            if let Some(ssid) = menu
                .select_ssid(
                    &mut self.station,
                    self.log_sender.clone(),
                    self.notification_sender.clone(),
                    icon_type,
                )
                .await?
            {
                let (network, _) = self
                    .station
                    .new_networks
                    .iter()
                    .chain(self.station.known_networks.iter())
                    .find(|(network, _)| network.name == ssid)
                    .unwrap();

                if self
                    .station
                    .connected_network
                    .as_ref()
                    .map_or(false, |cn| cn.name == ssid)
                {
                    self.station
                        .disconnect(self.log_sender.clone(), self.notification_sender.clone())
                        .await?;
                    self.station.refresh().await?;
                    continue;
                }

                if self
                    .station
                    .new_networks
                    .iter()
                    .any(|(n, _)| n.name == ssid)
                {
                    self.log_sender
                        .send(format!("Connecting to new network: {}", ssid))
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                }

                if let Some(known_network) = &network.known_network {
                    if known_network.is_autoconnect {
                        self.log_sender
                            .send(format!(
                                "Auto-connecting to known network: {}",
                                network.name
                            ))
                            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                        network
                            .connect(self.log_sender.clone(), self.notification_sender.clone())
                            .await?;
                        self.station.refresh().await?;
                        return Ok(Some(ssid));
                    }
                }

                if let Some(passphrase) = menu.prompt_passphrase(&network.name) {
                    self.agent_manager.send_passkey(passphrase)?;
                } else {
                    self.agent_manager.cancel_auth()?;
                }

                network
                    .connect(self.log_sender.clone(), self.notification_sender.clone())
                    .await?;
                self.station.refresh().await?;
            } else {
                self.log_sender
                    .send("No network selected".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                return Ok(None);
            }
        }
    }
}
