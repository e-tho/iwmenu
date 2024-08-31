use anyhow::Result;
use iwdrs::session::Session;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc::UnboundedSender, time::sleep};

use crate::{
    iw::{adapter::Adapter, agent::AgentManager, known_network::KnownNetwork},
    menu::Menu,
    notification::NotificationManager,
};

pub struct App {
    pub running: bool,
    pub reset_mode: bool,
    pub session: Arc<Session>,
    pub current_mode: String,
    adapter: Adapter,
    agent_manager: AgentManager,
    log_sender: UnboundedSender<String>,
    notification_manager: Arc<NotificationManager>,
}

impl App {
    pub async fn new(
        _menu: Menu,
        log_sender: UnboundedSender<String>,
        notification_manager: Arc<NotificationManager>,
    ) -> Result<Self> {
        let agent_manager = AgentManager::new().await?;
        let session = agent_manager.session();
        let adapter = Adapter::new(session.clone(), log_sender.clone()).await?;
        let current_mode = adapter.device.mode.clone();

        if !adapter.device.is_powered {
            adapter.device.power_on().await?;
        }

        Ok(Self {
            running: true,
            adapter,
            agent_manager,
            log_sender,
            notification_manager,
            session,
            current_mode,
            reset_mode: false,
        })
    }

    pub async fn reset(&mut self, mode: String, log_sender: UnboundedSender<String>) -> Result<()> {
        let session = match Session::new().await {
            Ok(session) => Arc::new(session),
            Err(e) => {
                self.log_sender
                    .send(format!("Failed to create a new session: {}", e))
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                return Err(e.into());
            }
        };

        let adapter = Adapter::new(session.clone(), log_sender.clone()).await?;
        adapter.device.set_mode(mode.clone()).await?;

        self.adapter = adapter;
        self.session = session;
        self.current_mode = mode;

        self.log_sender
            .send(format!("App state reset with mode: {}", self.current_mode))
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

        Ok(())
    }

    pub async fn run(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        if !self.adapter.device.is_powered {
            self.handle_device_off(menu, menu_command, icon_type, spaces)
                .await?;
            self.adapter.refresh(self.log_sender.clone()).await?;
        }

        while self.running {
            self.adapter.refresh(self.log_sender.clone()).await?;

            match self.adapter.device.mode.as_str() {
                "station" => {
                    if let Some(station) = self.adapter.device.station.as_mut() {
                        let output = menu
                            .show_main_menu(menu_command, station, icon_type, spaces)
                            .await?;

                        if let Some(output) = output {
                            match output.as_str() {
                                o if o.contains("Scan") => self.handle_scan().await?,
                                o if o.contains("Known Networks") => {
                                    if let Some(known_network) = menu
                                        .show_known_networks_menu(
                                            menu_command,
                                            station,
                                            icon_type,
                                            spaces,
                                        )
                                        .await?
                                    {
                                        self.handle_known_network_options(
                                            menu,
                                            menu_command,
                                            &known_network,
                                            icon_type,
                                            spaces,
                                        )
                                        .await?;
                                    }
                                }
                                o if o.contains("Settings") => {
                                    self.handle_settings(menu, menu_command, icon_type, spaces)
                                        .await?;
                                }
                                _ => {
                                    if let Some(ssid) = self
                                        .handle_network_selection(
                                            menu,
                                            menu_command,
                                            &output,
                                            icon_type,
                                            spaces,
                                        )
                                        .await?
                                    {
                                        return Ok(Some(ssid));
                                    }
                                }
                            }
                        } else {
                            self.log_sender
                                .send("No network selected".to_string())
                                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                            self.running = false;
                            return Ok(None);
                        }
                    } else {
                        self.log_sender
                            .send("No station available".to_string())
                            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                        self.running = false;
                        return Ok(None);
                    }
                }
                "ap" => {
                    self.handle_ap_menu(menu, menu_command, icon_type, spaces)
                        .await?;
                }
                _ => {
                    self.log_sender
                        .send("Unknown mode".to_string())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    self.running = false;
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    async fn handle_device_off(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        if let Some(output) = menu.prompt_enable_adapter(menu_command, icon_type, spaces) {
            if output.contains("Power On Device") {
                self.adapter.device.power_on().await?;
                self.log_sender
                    .send("Adapter enabled".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                self.notification_manager.send_notification(
                    None,
                    Some("Adapter enabled".to_string()),
                    None,
                    None,
                );

                self.adapter.refresh(self.log_sender.clone()).await?;

                if let Some(station) = self.adapter.device.station.as_mut() {
                    if station.is_scanning {
                        self.log_sender
                            .send("Waiting for ongoing scan to complete...".to_string())
                            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                        while station.is_scanning {
                            station.refresh().await?;
                            sleep(Duration::from_millis(500)).await;
                        }
                    }

                    station
                        .scan(
                            self.log_sender.clone(),
                            Arc::clone(&self.notification_manager),
                        )
                        .await?;
                    station.refresh().await?;
                }
            } else {
                self.log_sender
                    .send("Adapter remains disabled".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                self.notification_manager.send_notification(
                    None,
                    Some("Adapter remains disabled".to_string()),
                    None,
                    None,
                );
            }
        }

        Ok(())
    }

    async fn handle_scan(&mut self) -> Result<()> {
        if let Some(station) = self.adapter.device.station.as_mut() {
            station
                .scan(
                    self.log_sender.clone(),
                    Arc::clone(&self.notification_manager),
                )
                .await?;
            station.refresh().await?;
        }
        Ok(())
    }

    async fn handle_known_network_options(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        known_network: &KnownNetwork,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        if let Some(option) = menu
            .show_known_network_options(menu_command, known_network, icon_type, spaces)
            .await?
        {
            match option.as_str() {
                opt if opt.contains("Disable Autoconnect")
                    || opt.contains("Enable Autoconnect") =>
                {
                    known_network
                        .toggle_autoconnect(
                            self.log_sender.clone(),
                            self.notification_manager.clone(),
                        )
                        .await?;
                    if let Some(station) = self.adapter.device.station.as_mut() {
                        station.refresh().await?;
                    }
                }
                opt if opt.contains("Forget Network") => {
                    known_network
                        .forget(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                    if let Some(station) = self.adapter.device.station.as_mut() {
                        station.refresh().await?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_network_selection(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        output: &str,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        if let Some(station) = self.adapter.device.station.as_mut() {
            let networks = station
                .new_networks
                .iter()
                .chain(station.known_networks.iter());

            if let Some((network, _)) =
                menu.select_network(networks, output.to_string(), icon_type, spaces)
            {
                if station
                    .connected_network
                    .as_ref()
                    .map_or(false, |cn| cn.name == network.name)
                {
                    station
                        .disconnect(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                    station.refresh().await?;
                    return Ok(None);
                }

                if station
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
                            .connect(self.log_sender.clone(), self.notification_manager.clone())
                            .await?;
                        return Ok(Some(network.name.clone()));
                    }
                }

                if let Some(passphrase) =
                    menu.prompt_passphrase(menu_command, &network.name, icon_type)
                {
                    self.agent_manager.send_passkey(passphrase)?;
                } else {
                    self.agent_manager.cancel_auth()?;
                }

                network
                    .connect(self.log_sender.clone(), self.notification_manager.clone())
                    .await?;
                station.refresh().await?;
                return Ok(Some(network.name.clone()));
            }
        }
        Ok(None)
    }

    async fn handle_settings(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        let input = menu.get_settings_icons(icon_type, spaces);

        if let Some(output) = menu.run_menu_command(menu_command, &input, icon_type) {
            match output.as_str() {
                o if o.contains("Disable Adapter") => {
                    self.disable_adapter(menu, menu_command, icon_type, spaces)
                        .await?
                }
                o if o.contains("Change Mode") => {
                    self.handle_change_mode(menu, menu_command, icon_type, spaces)
                        .await?
                }
                _ if self.adapter.device.mode == "ap" => {
                    self.handle_ap_menu(menu, menu_command, icon_type, spaces)
                        .await?
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn disable_adapter(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        self.adapter.device.power_off().await?;
        self.log_sender
            .send("Adapter disabled".to_string())
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
        self.notification_manager.send_notification(
            None,
            Some("Adapter disabled".to_string()),
            None,
            None,
        );

        self.handle_device_off(menu, menu_command, icon_type, spaces)
            .await?;

        Ok(())
    }

    async fn handle_change_mode(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        if let Ok(Some(output)) =
            menu.show_change_mode_menu(menu_command, &self.adapter, icon_type, spaces)
        {
            if self.adapter.supported_modes.contains(&output) {
                self.reset(output, self.log_sender.clone()).await?;
                self.reset_mode = true;
                self.running = false;
            }
        }
        Ok(())
    }

    async fn handle_ap_menu(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        loop {
            if let Some(ap) = self.adapter.device.access_point.as_ref() {
                if let Ok(Some(output)) = menu.show_ap_menu(menu_command, ap, icon_type, spaces) {
                    match output.trim() {
                        o if o.contains("Start AP") => {
                            self.start_ap(menu, menu_command, icon_type).await?
                        }
                        o if o.contains("Stop AP") => self.stop_ap().await?,
                        o if o.contains("Set SSID") => {
                            if let Some(ssid) = menu.prompt_ssid(menu_command, icon_type) {
                                if let Some(ap) = self.adapter.device.access_point.as_mut() {
                                    ap.ssid = ssid.clone();
                                    self.log_sender
                                        .send(format!("SSID set to {}", ssid))
                                        .unwrap_or_else(|err| {
                                            println!("Failed to send message: {}", err)
                                        });
                                }
                            }
                        }
                        o if o.contains("Set Password") => {
                            if let Some(password) = menu.prompt_password(menu_command, icon_type) {
                                if let Some(ap) = self.adapter.device.access_point.as_mut() {
                                    ap.psk = password;
                                    self.log_sender
                                        .send("Password set".to_string())
                                        .unwrap_or_else(|err| {
                                            println!("Failed to send message: {}", err)
                                        });
                                }
                            }
                        }
                        o if o.contains("Change Mode") => {
                            self.handle_change_mode(menu, menu_command, icon_type, spaces)
                                .await?;
                            if self.reset_mode {
                                break;
                            }
                        }
                        _ => {
                            self.running = false;
                            self.reset_mode = false;
                            break;
                        }
                    }
                } else {
                    self.running = false;
                    break;
                }
            } else {
                self.log_sender
                    .send("No access point available".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                self.reset("station".to_string(), self.log_sender.clone())
                    .await?;
                self.reset_mode = true;
                self.running = false;
                break;
            }

            if self.reset_mode {
                break;
            }
        }

        Ok(())
    }

    async fn start_ap(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
    ) -> Result<()> {
        if let Some(ap) = self.adapter.device.access_point.as_mut() {
            if ap.has_started {
                self.log_sender
                    .send("Access Point is already started".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                return Ok(());
            }

            let ssid = menu
                .prompt_ssid(menu_command, icon_type)
                .unwrap_or_else(|| "MySSID".to_string());
            let psk = menu
                .prompt_password(menu_command, icon_type)
                .unwrap_or_else(|| "MyPassword".to_string());

            ap.set_ssid(ssid);
            ap.set_psk(psk);

            match ap.start().await {
                Ok(_) => {
                    self.log_sender
                        .send("Access Point started successfully".to_string())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    self.notification_manager.send_notification(
                        None,
                        Some("Access Point started successfully".to_string()),
                        None,
                        None,
                    );
                }
                Err(e) => {
                    self.log_sender
                        .send(format!("Failed to start Access Point: {}", e))
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    self.notification_manager.send_notification(
                        None,
                        Some(format!("Failed to start Access Point: {}", e)),
                        None,
                        None,
                    );
                }
            }

            self.adapter.refresh(self.log_sender.clone()).await?;
        } else {
            self.log_sender
                .send("No Access Point available to start".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            self.notification_manager.send_notification(
                None,
                Some("No Access Point available to start".to_string()),
                None,
                None,
            );
        }

        Ok(())
    }

    async fn stop_ap(&mut self) -> Result<()> {
        if let Some(ap) = &self.adapter.device.access_point {
            ap.stop().await?;
            self.adapter.refresh(self.log_sender.clone()).await?;
            self.log_sender
                .send("Access Point stopped".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            self.notification_manager.send_notification(
                None,
                Some("Access Point stopped".to_string()),
                None,
                None,
            );
        }
        Ok(())
    }
}
