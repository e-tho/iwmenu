use anyhow::Result;
use notify_rust::Timeout;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    iw::{agent::AgentManager, known_network::KnownNetwork, station::Station},
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

    pub async fn run(&mut self, menu: &Menu, icon_type: &str) -> Result<Option<String>> {
        loop {
            if let Some(output) = menu.show_menu(&mut self.station, icon_type).await? {
                if output.contains("Scan") {
                    self.handle_scan().await?;
                } else if output.contains("Known Networks") {
                    if let Some(known_network) = menu
                        .show_known_networks_menu(&mut self.station, icon_type)
                        .await?
                    {
                        self.handle_known_network_options(menu, &known_network, icon_type)
                            .await?;
                    }
                } else if let Some(ssid) = self
                    .handle_network_selection(menu, &output, icon_type)
                    .await?
                {
                    return Ok(Some(ssid));
                }
            } else {
                self.log_sender
                    .send("No network selected".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                return Ok(None);
            }
        }
    }

    async fn handle_scan(&mut self) -> Result<()> {
        self.station
            .scan(self.log_sender.clone(), self.notification_sender.clone())
            .await?;
        self.station.refresh().await?;
        Ok(())
    }

    async fn handle_known_network_options(
        &mut self,
        menu: &Menu,
        known_network: &KnownNetwork,
        icon_type: &str,
    ) -> Result<()> {
        if let Some(option) = menu
            .show_known_network_options(known_network, icon_type)
            .await?
        {
            match option.as_str() {
                "Toggle Autoconnect" => {
                    known_network
                        .toggle_autoconnect(
                            self.log_sender.clone(),
                            self.notification_sender.clone(),
                        )
                        .await?;
                }
                "Forget Network" => {
                    known_network
                        .forget(self.log_sender.clone(), self.notification_sender.clone())
                        .await?;
                    self.station.refresh().await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_network_selection(
        &mut self,
        menu: &Menu,
        output: &str,
        icon_type: &str,
    ) -> Result<Option<String>> {
        let networks = self
            .station
            .new_networks
            .iter()
            .chain(self.station.known_networks.iter());

        if let Some((network, _)) = menu.select_network(networks, output.to_string(), icon_type) {
            if self
                .station
                .connected_network
                .as_ref()
                .map_or(false, |cn| cn.name == network.name)
            {
                self.station
                    .disconnect(self.log_sender.clone(), self.notification_sender.clone())
                    .await?;
                self.station.refresh().await?;
                return Ok(None);
            }

            if self
                .station
                .new_networks
                .iter()
                .any(|(n, _)| n.name == network.name)
            {
                self.log_sender
                    .send(format!("Connecting to new network: {}", network.name))
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
                    return Ok(Some(network.name.clone()));
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
            Ok(Some(network.name.clone()))
        } else {
            Ok(None)
        }
    }
}
