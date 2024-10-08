use crate::{
    iw::{adapter::Adapter, agent::AgentManager, known_network::KnownNetwork},
    menu::{
        AdapterMenuOptions, ApMenuOptions, ChangeModeMenuOptions, KnownNetworkOptions,
        MainMenuOptions, Menu, SettingsMenuOptions,
    },
    notification::NotificationManager,
};
use anyhow::Result;
use iwdrs::{modes::Mode, session::Session};
use rust_i18n::t;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct App {
    pub running: bool,
    pub reset_mode: bool,
    pub session: Arc<Session>,
    pub current_mode: Mode,
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

    pub async fn reset(&mut self, mode: Mode, log_sender: UnboundedSender<String>) -> Result<()> {
        let session = Arc::new(Session::new().await?);
        let adapter = Adapter::new(session.clone(), log_sender.clone()).await?;
        adapter.device.set_mode(mode.clone()).await?;

        self.adapter = adapter;
        self.session = session;
        self.current_mode = mode;

        self.log_sender
            .send(format!(
                "App state reset with mode: {:?}",
                self.current_mode
            ))
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
        }

        while self.running {
            self.adapter.refresh(self.log_sender.clone()).await?;

            match self.adapter.device.mode {
                Mode::Station => {
                    if let Some(station) = self.adapter.device.station.as_mut() {
                        if let Some(main_menu_option) = menu
                            .show_main_menu(menu_command, station, icon_type, spaces)
                            .await?
                        {
                            match main_menu_option {
                                MainMenuOptions::Scan => {
                                    self.handle_scan().await?;
                                }
                                MainMenuOptions::KnownNetworks => {
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
                                MainMenuOptions::Settings => {
                                    self.handle_settings(menu, menu_command, icon_type, spaces)
                                        .await?;
                                }
                                MainMenuOptions::Network(output) => {
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
                                .send(t!("notifications.app.no_network_selected").to_string())
                                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                            self.running = false;
                            return Ok(None);
                        }
                    } else {
                        self.log_sender
                            .send(t!("notifications.app.no_station_available").to_string())
                            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                        self.running = false;
                        return Ok(None);
                    }
                }
                Mode::Ap => {
                    self.handle_ap_menu(menu, menu_command, icon_type, spaces)
                        .await?;
                }
                _ => {
                    self.log_sender
                        .send(t!("notifications.app.unknown_mode").to_string())
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
        if let Some(option) = menu.prompt_enable_adapter(menu_command, icon_type, spaces) {
            match option {
                AdapterMenuOptions::PowerOnDevice => {
                    self.adapter.device.power_on().await?;
                    self.log_sender
                        .send(t!("notifications.app.adapter_enabled").to_string())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    self.notification_manager.send_notification(
                        None,
                        Some(t!("notifications.app.adapter_enabled").to_string()),
                        None,
                        None,
                    );

                    self.adapter.refresh(self.log_sender.clone()).await?;

                    if let Some(station) = self.adapter.device.station.as_mut() {
                        station
                            .scan(
                                self.log_sender.clone(),
                                Arc::clone(&self.notification_manager),
                            )
                            .await?;
                        station.refresh(self.log_sender.clone()).await?;
                    }
                }
            }
        } else {
            self.log_sender
                .send(t!("notifications.app.adapter_disabled").to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            self.notification_manager.send_notification(
                None,
                Some(t!("notifications.app.adapter_disabled").to_string()),
                None,
                None,
            );
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
            match option {
                KnownNetworkOptions::DisableAutoconnect
                | KnownNetworkOptions::EnableAutoconnect => {
                    known_network
                        .toggle_autoconnect(
                            self.log_sender.clone(),
                            self.notification_manager.clone(),
                        )
                        .await?;
                }
                KnownNetworkOptions::ForgetNetwork => {
                    known_network
                        .forget(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                }
            }
        }
        if let Some(station) = self.adapter.device.station.as_mut() {
            station.refresh(self.log_sender.clone()).await?;
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
                    station.refresh(self.log_sender.clone()).await?;
                    return Ok(None);
                }

                if network.known_network.is_some() {
                    self.log_sender
                        .send(format!("Connecting to known network: {}", network.name))
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    network
                        .connect(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                    station.refresh(self.log_sender.clone()).await?;
                    return Ok(Some(network.name.clone()));
                } else {
                    self.log_sender
                        .send(format!("Connecting to new network: {}", network.name))
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    if let Some(passphrase) =
                        menu.prompt_passphrase(menu_command, &network.name, icon_type)
                    {
                        self.agent_manager.send_passkey(passphrase)?;
                    } else {
                        self.agent_manager.cancel_auth()?;
                        return Ok(None);
                    }

                    network
                        .connect(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                    station.refresh(self.log_sender.clone()).await?;
                    return Ok(Some(network.name.clone()));
                }
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
        if let Some(option) = menu
            .show_settings_menu(menu_command, &self.current_mode, icon_type, spaces)
            .await?
        {
            match option {
                SettingsMenuOptions::DisableAdapter => {
                    self.disable_adapter(menu, menu_command, icon_type, spaces)
                        .await?;
                }
                SettingsMenuOptions::SwitchMode => {
                    self.switch_mode().await?;
                    self.reset_mode = true;
                    self.running = false;
                }
            }
        }

        Ok(())
    }

    async fn switch_mode(&mut self) -> Result<()> {
        let new_mode = match self.current_mode {
            Mode::Station => Mode::Ap,
            Mode::Ap => Mode::Station,
            _ => Mode::Station, // Valeur par défaut
        };

        self.reset(new_mode, self.log_sender.clone()).await?;

        self.log_sender
            .send(format!("Switched to mode: {:?}", self.current_mode))
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

        self.notification_manager.send_notification(
            None,
            Some(format!("Switched to mode: {:?}", self.current_mode)),
            None,
            None,
        );

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
            .send(t!("notifications.app.adapter_disabled").to_string())
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
        self.notification_manager.send_notification(
            None,
            Some(t!("notifications.app.adapter_disabled").to_string()),
            None,
            None,
        );

        self.handle_device_off(menu, menu_command, icon_type, spaces)
            .await?;

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
            if let Some(ap) = self.adapter.device.access_point.as_mut() {
                if let Ok(Some(ap_menu_option)) =
                    menu.show_ap_menu(menu_command, ap, icon_type, spaces).await
                {
                    match ap_menu_option {
                        ApMenuOptions::StartAp => {
                            if ap.ssid.is_empty() || ap.psk.is_empty() {
                                self.log_sender
                                    .send("SSID or Password not set".to_string())
                                    .unwrap_or_else(|err| {
                                        println!("Failed to send message: {}", err)
                                    });
                                if ap.ssid.is_empty() {
                                    if let Some(ssid) = menu.prompt_ssid(menu_command, icon_type) {
                                        ap.set_ssid(ssid);
                                    }
                                }
                                if ap.psk.is_empty() {
                                    if let Some(password) =
                                        menu.prompt_password(menu_command, icon_type)
                                    {
                                        ap.set_psk(password);
                                    }
                                }
                            }
                            if !ap.ssid.is_empty() && !ap.psk.is_empty() {
                                self.start_ap(menu, menu_command, icon_type).await?;
                            }
                        }
                        ApMenuOptions::StopAp => self.stop_ap().await?,
                        ApMenuOptions::SetSsid => {
                            if let Some(ssid) = menu.prompt_ssid(menu_command, icon_type) {
                                ap.set_ssid(ssid.clone());
                                self.log_sender
                                    .send(format!("SSID set to {}", ssid))
                                    .unwrap_or_else(|err| {
                                        println!("Failed to send message: {}", err)
                                    });
                            }
                        }
                        ApMenuOptions::SetPassword => {
                            if let Some(password) = menu.prompt_password(menu_command, icon_type) {
                                ap.set_psk(password.clone());
                                self.log_sender
                                    .send("Password set".to_string())
                                    .unwrap_or_else(|err| {
                                        println!("Failed to send message: {}", err)
                                    });
                            }
                        }
                        ApMenuOptions::Settings => {
                            self.handle_settings(menu, menu_command, icon_type, spaces)
                                .await?;
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
                self.reset(Mode::Station, self.log_sender.clone()).await?;
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

            let ssid = if ap.ssid.is_empty() {
                menu.prompt_ssid(menu_command, icon_type)
                    .unwrap_or_else(|| "MySSID".to_string())
            } else {
                ap.ssid.clone()
            };

            let psk = if ap.psk.is_empty() {
                menu.prompt_password(menu_command, icon_type)
                    .unwrap_or_else(|| "MyPassword".to_string())
            } else {
                ap.psk.clone()
            };

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
