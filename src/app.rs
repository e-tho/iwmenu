use crate::{
    iw::{adapter::Adapter, agent::AgentManager, known_network::KnownNetwork, network::Network},
    menu::{
        AdapterMenuOptions, ApMenuOptions, KnownNetworkOptions, MainMenuOptions, Menu,
        SettingsMenuOptions,
    },
    notification::NotificationManager,
};
use anyhow::Result;
use iwdrs::{modes::Mode, session::Session};
use notify_rust::Timeout;
use rust_i18n::t;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc::UnboundedSender, time::sleep};

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

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub async fn run(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        if !self.adapter.device.is_powered {
            self.handle_adapter_options(menu, menu_command, icon_type, spaces)
                .await?;
        }

        while self.running {
            self.adapter.refresh().await?;

            match self.adapter.device.mode {
                Mode::Station => {
                    let ssid = {
                        if let Some(station) = self.adapter.device.station.as_mut() {
                            if let Some(main_menu_option) = menu
                                .show_main_menu(menu_command, station, icon_type, spaces)
                                .await?
                            {
                                self.handle_main_options(
                                    menu,
                                    menu_command,
                                    icon_type,
                                    spaces,
                                    main_menu_option,
                                )
                                .await?
                            } else {
                                self.log_sender
                                    .send(t!("notifications.app.no_network_selected").to_string())
                                    .unwrap_or_else(|err| {
                                        println!("Failed to send message: {}", err)
                                    });
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
                    };

                    if let Some(ssid) = ssid {
                        return Ok(Some(ssid));
                    }
                }
                Mode::Ap => {
                    if let Some(ap_menu_option) = menu
                        .show_ap_menu(
                            menu_command,
                            self.adapter.device.access_point.as_mut().unwrap(),
                            icon_type,
                            spaces,
                        )
                        .await?
                    {
                        self.handle_ap_options(
                            ap_menu_option,
                            menu,
                            menu_command,
                            icon_type,
                            spaces,
                        )
                        .await?;
                    } else {
                        self.perform_mode_switch(menu).await?;
                    }
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

    async fn handle_main_options(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
        main_menu_option: MainMenuOptions,
    ) -> Result<Option<String>> {
        match main_menu_option {
            MainMenuOptions::Scan => {
                self.perform_network_scan().await?;
            }
            MainMenuOptions::Settings => {
                if let Some(option) = menu
                    .show_settings_menu(menu_command, &self.current_mode, icon_type, spaces)
                    .await?
                {
                    self.handle_settings_options(option, menu, menu_command, icon_type, spaces)
                        .await?;
                }
            }
            MainMenuOptions::Network(output) => {
                if let Some(ssid) = self
                    .handle_network_selection(menu, menu_command, &output, icon_type, spaces)
                    .await?
                {
                    return Ok(Some(ssid));
                }
            }
        }
        Ok(None)
    }

    async fn handle_ap_options(
        &mut self,
        ap_menu_option: ApMenuOptions,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        if let Some(ap) = self.adapter.device.access_point.as_mut() {
            match ap_menu_option {
                ApMenuOptions::StartAp => {
                    if ap.ssid.is_empty() || ap.psk.is_empty() {
                        self.log_sender
                            .send("SSID or Password not set".to_string())
                            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                        if ap.ssid.is_empty() {
                            if let Some(ssid) = menu.prompt_ap_ssid(menu_command, icon_type) {
                                ap.set_ssid(ssid);
                            }
                        }
                        if ap.psk.is_empty() {
                            if let Some(password) =
                                menu.prompt_ap_passphrase(menu_command, icon_type)
                            {
                                ap.set_psk(password);
                            }
                        }
                    }
                    if !ap.ssid.is_empty() && !ap.psk.is_empty() {
                        self.perform_ap_start(menu, menu_command, icon_type).await?;
                    }
                }
                ApMenuOptions::StopAp => self.perform_ap_stop().await?,
                ApMenuOptions::SetSsid => {
                    if let Some(ssid) = menu.prompt_ap_ssid(menu_command, icon_type) {
                        ap.set_ssid(ssid.clone());
                        self.log_sender
                            .send(format!("SSID set to {}", ssid))
                            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    }
                }
                ApMenuOptions::SetPassword => {
                    if let Some(password) = menu.prompt_ap_passphrase(menu_command, icon_type) {
                        ap.set_psk(password.clone());
                        self.log_sender
                            .send("Password set".to_string())
                            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    }
                }
                ApMenuOptions::Settings => {
                    if let Some(option) = menu
                        .show_settings_menu(menu_command, &self.current_mode, icon_type, spaces)
                        .await?
                    {
                        self.handle_settings_options(option, menu, menu_command, icon_type, spaces)
                            .await?;
                    }
                }
            }
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
        is_connected: bool,
    ) -> Result<()> {
        let mut available_options = vec![];

        if is_connected {
            available_options.push(KnownNetworkOptions::Disconnect);
        } else {
            available_options.push(KnownNetworkOptions::Connect);
        }

        available_options.push(KnownNetworkOptions::ForgetNetwork);
        available_options.push(if known_network.is_autoconnect {
            KnownNetworkOptions::DisableAutoconnect
        } else {
            KnownNetworkOptions::EnableAutoconnect
        });

        if let Some(option) = menu
            .show_known_network_options(menu_command, icon_type, spaces, available_options)
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
                KnownNetworkOptions::Disconnect => {
                    if is_connected {
                        self.perform_network_disconnection().await?;
                    }
                }
                KnownNetworkOptions::Connect => {
                    if let Some(station) = self.adapter.device.station.as_mut() {
                        if let Some(network) = station
                            .known_networks
                            .iter()
                            .find(|(net, _)| net.name == known_network.name)
                            .map(|(net, _)| net.clone())
                        {
                            self.perform_known_network_connection(&network).await?;
                        }
                    }
                }
            }
        }

        if let Some(station) = self.adapter.device.station.as_mut() {
            station.refresh().await?;
        }

        Ok(())
    }

    async fn handle_settings_options(
        &mut self,
        option: SettingsMenuOptions,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        match option {
            SettingsMenuOptions::DisableAdapter => {
                self.perform_adapter_disable(menu, menu_command, icon_type, spaces)
                    .await?;
            }
            SettingsMenuOptions::SwitchMode => {
                self.perform_mode_switch(menu).await?;
                self.reset_mode = true;
                self.running = false;
            }
        }
        Ok(())
    }

    async fn handle_adapter_options(
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
                    self.adapter.refresh().await?;
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
                if let Some(ref known_network) = network.known_network {
                    let is_connected = station
                        .connected_network
                        .as_ref()
                        .map_or(false, |cn| cn.name == network.name);

                    self.handle_known_network_options(
                        menu,
                        menu_command,
                        known_network,
                        icon_type,
                        spaces,
                        is_connected,
                    )
                    .await?;
                    return Ok(None);
                } else {
                    return self
                        .perform_new_network_connection(menu, menu_command, &network, icon_type)
                        .await;
                }
            }
        }
        Ok(None)
    }

    async fn perform_known_network_connection(
        &mut self,
        network: &Network,
    ) -> Result<Option<String>> {
        let station = self.adapter.device.station.as_mut().unwrap();

        self.log_sender
            .send(format!("Connecting to known network: {}", network.name))
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

        network
            .connect(self.log_sender.clone(), self.notification_manager.clone())
            .await?;

        station.refresh().await?;

        Ok(Some(network.name.clone()))
    }

    async fn perform_new_network_connection(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        network: &Network,
        icon_type: &str,
    ) -> Result<Option<String>> {
        let station = self.adapter.device.station.as_mut().unwrap();

        self.log_sender
            .send(format!("Connecting to new network: {}", network.name))
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

        if let Some(passphrase) =
            menu.prompt_station_passphrase(menu_command, &network.name, icon_type)
        {
            self.agent_manager.send_passkey(passphrase)?;
        } else {
            self.agent_manager.cancel_auth()?;
            return Ok(None);
        }

        network
            .connect(self.log_sender.clone(), self.notification_manager.clone())
            .await?;

        station.refresh().await?;

        Ok(Some(network.name.clone()))
    }

    async fn perform_network_disconnection(&mut self) -> Result<()> {
        let station = self.adapter.device.station.as_mut().unwrap();

        if let Some(connected_network) = &station.connected_network {
            self.log_sender
                .send(format!(
                    "Disconnecting from network: {}",
                    connected_network.name
                ))
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
        } else {
            self.log_sender
                .send("No network is currently connected.".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
        }

        station
            .disconnect(self.log_sender.clone(), self.notification_manager.clone())
            .await?;
        station.refresh().await?;

        Ok(())
    }

    async fn perform_network_scan(&mut self) -> Result<()> {
        if let Some(station) = self.adapter.device.station.as_mut() {
            if station.is_scanning {
                let msg = t!("notifications.station.scan_already_in_progress");
                self.log_sender
                    .send(msg.to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                self.notification_manager.send_notification(
                    None,
                    Some(msg.to_string()),
                    None,
                    None,
                );
                return Ok(());
            }

            if let Err(e) = station.scan().await {
                let err_msg = format!("Failed to initiate network scan: {}", e);
                self.log_sender.send(err_msg.clone()).ok();
                self.notification_manager.send_notification(
                    None,
                    Some(t!("notifications.station.scan_failed").to_string()),
                    None,
                    None,
                );
                return Err(e.into());
            }

            let handle = self.notification_manager.send_notification(
                None,
                Some(t!("notifications.station.scan_in_progress").to_string()),
                None,
                Some(Timeout::Never),
            );

            while station.is_scanning {
                sleep(Duration::from_millis(500)).await;
            }

            station.refresh().await?;

            handle.close();

            self.log_sender
                .send("Scan completed".to_string())
                .unwrap_or_else(|err| println!("Log error: {}", err));
        }

        Ok(())
    }

    async fn perform_mode_switch(&mut self, menu: &Menu) -> Result<()> {
        let new_mode = match self.current_mode {
            Mode::Station => Mode::Ap,
            Mode::Ap => Mode::Station,
            _ => Mode::Station,
        };

        self.reset(new_mode, self.log_sender.clone()).await?;

        let mode_text = menu.get_mode_text(&self.current_mode);
        let msg = t!("notifications.device.switched_mode", mode = mode_text).to_string();

        self.log_sender
            .send(msg.clone())
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

        self.notification_manager
            .send_notification(None, Some(msg), None, None);

        Ok(())
    }

    async fn perform_adapter_disable(
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

        self.handle_adapter_options(menu, menu_command, icon_type, spaces)
            .await?;

        Ok(())
    }

    async fn perform_ap_start(
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
                menu.prompt_ap_ssid(menu_command, icon_type)
                    .unwrap_or_else(|| "MySSID".to_string())
            } else {
                ap.ssid.clone()
            };

            let psk = if ap.psk.is_empty() {
                menu.prompt_ap_passphrase(menu_command, icon_type)
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
                        Some(t!("notifications.device.access_point_started").to_string()),
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
                        Some(t!("notifications.device.access_point_start_failed", error = e.to_string()).to_string()),
                        None,
                        None,
                    );
                }
            }

            self.adapter.refresh().await?;
        } else {
            self.log_sender
                .send("No Access Point available to start".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            self.notification_manager.send_notification(
                None,
                Some(t!("notifications.device.no_access_point_available").to_string()),
                None,
                None,
            );
        }

        Ok(())
    }

    async fn perform_ap_stop(&mut self) -> Result<()> {
        if let Some(ap) = &self.adapter.device.access_point {
            ap.stop().await?;
            self.adapter.refresh().await?;
            self.log_sender
                .send("Access Point stopped".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            self.notification_manager.send_notification(
                None,
                Some(t!("notifications.device.access_point_stopped").to_string()),
                None,
                None,
            );
        }
        Ok(())
    }
}
