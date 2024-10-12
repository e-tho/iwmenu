use crate::{
    iw::{adapter::Adapter, agent::AgentManager, known_network::KnownNetwork},
    menu::{
        AdapterMenuOptions, ApMenuOptions, KnownNetworkOptions, MainMenuOptions, Menu, MenuProcess,
        SettingsMenuOptions,
    },
    notification::NotificationManager,
};
use anyhow::Result;
use iwdrs::{modes::Mode, session::Session};
use rust_i18n::t;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub struct App {
    pub running: bool,
    pub reset_mode: bool,
    pub session: Arc<Session>,
    pub current_mode: Mode,
    adapter: Adapter,
    agent_manager: AgentManager,
    log_sender: UnboundedSender<String>,
    notification_manager: Arc<NotificationManager>,
    pub current_menu: CurrentMenu,
    current_menu_process: Option<MenuProcess>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurrentMenu {
    MainMenu,
    KnownNetworksMenu,
    SettingsMenu,
    ApMenu,
    EnableAdapterMenu,
}

impl App {
    pub async fn new(
        menu: Menu,
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
            current_menu_process: None,
            current_menu: CurrentMenu::MainMenu,
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
        let (scan_complete_tx, mut scan_complete_rx) = unbounded_channel();

        if !self.adapter.device.is_powered {
            self.handle_device_off(
                menu,
                menu_command,
                icon_type,
                spaces,
                scan_complete_tx.clone(),
            )
            .await?;
        }

        while self.running {
            self.adapter.refresh(self.log_sender.clone()).await?;

            match self.adapter.device.mode {
                Mode::Station => {
                    tokio::select! {
                        _ = scan_complete_rx.recv() => {
                            self.adapter.refresh(self.log_sender.clone()).await?;
                            self.log_sender.send("Scan terminé.".to_string())
                                .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));

                            if matches!(self.current_menu, CurrentMenu::MainMenu) {
                                self.close_menu_process().await;
                            } else {
                                self.log_sender
                                    .send(format!("Le menu actuel n'est pas le menu principal: {:?}", self.current_menu))
                                    .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));

                                continue;
                            }
                            continue;
                        }



                        result = self.handle_main_menu(menu, menu_command, icon_type, spaces, scan_complete_tx.clone()) => {
                            match result {
                                Ok(Some(_)) => {
                                    self.log_sender.send("Menu principal traité avec succès.".to_string())
                                        .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));
                                }
                                Ok(None) => {
                                    self.log_sender.send("L'utilisateur a quitté le menu ou n'a rien sélectionné.".to_string())
                                        .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));
                                    self.quit();
                                    break;
                                }
                                Err(err) => {
                                    self.log_sender.send(format!("Erreur lors du traitement du menu: {}", err))
                                        .unwrap_or_else(|_| println!("Échec de l'envoi du message"));
                                    self.quit();
                                    break;
                                }
                            }
                        }
                    }
                }
                Mode::Ap => {
                    self.handle_ap_menu(
                        menu,
                        menu_command,
                        icon_type,
                        spaces,
                        scan_complete_tx.clone(),
                    )
                    .await?;
                }
                _ => {
                    self.log_sender
                        .send(t!("notifications.app.unknown_mode").to_string())
                        .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));
                    self.quit();
                    break;
                }
            }
        }

        Ok(None)
    }

    async fn close_menu_process(&mut self) {
        if !matches!(self.current_menu, CurrentMenu::MainMenu) {
            return;
        }

        if let Some(menu_process) = &mut self.current_menu_process {
            let child = &mut menu_process.child;

            if let Ok(Some(_)) = child.try_wait() {
            } else {
                if let Some(child_id) = child.id() {
                    let pid = nix::unistd::Pid::from_raw(child_id as i32);

                    if let Err(e) = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM) {
                        eprintln!("Erreur lors de l'envoi du signal SIGTERM : {:?}", e);
                    }

                    let _ = child.wait().await;
                }
            }
        }

        self.current_menu_process = None;
    }

    pub async fn handle_main_menu(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
        scan_complete_tx: UnboundedSender<()>,
    ) -> Result<Option<String>> {
        self.current_menu = CurrentMenu::MainMenu;

        let main_menu_option = {
            if let Some(station_arc) = self.adapter.device.station.as_ref() {
                let mut station_guard = station_arc.write().await;

                if let Ok(Some(menu_process)) = menu
                    .show_main_menu(menu_command, &mut *station_guard, icon_type, spaces)
                    .await
                {
                    self.current_menu_process = Some(menu_process);

                    let output = self
                        .current_menu_process
                        .as_mut()
                        .unwrap()
                        .output_future
                        .as_mut()
                        .await;
                    if output.is_none() {
                        self.log_sender
                            .send(t!("notifications.app.no_selection_or_quit").to_string())
                            .unwrap_or_else(|err| {
                                println!("Échec de l'envoi du message : {}", err)
                            });

                        if !matches!(self.current_menu, CurrentMenu::MainMenu) {
                            self.log_sender
                                .send("Le menu actuel n'est pas le menu principal, continuer l'exécution.".to_string())
                                .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));

                            return Ok(Some("menu_continue".to_string()));
                        }

                        return Ok(None);
                    }

                    if let Some(output) = output {
                        let cleaned_output = menu.clean_menu_output(&output, icon_type);
                        MainMenuOptions::from_str(&cleaned_output)
                    } else {
                        self.log_sender
                            .send(t!("notifications.app.no_network_selected").to_string())
                            .unwrap_or_else(|err| {
                                println!("Échec de l'envoi du message : {}", err)
                            });
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            } else {
                self.log_sender
                    .send(t!("notifications.app.no_station_available").to_string())
                    .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));
                self.running = false;
                return Ok(None);
            }
        };

        if let Some(option) = main_menu_option {
            match option {
                MainMenuOptions::Scan => {
                    self.log_sender
                        .send(t!("notifications.app.starting_scan").to_string())
                        .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));

                    self.handle_scan(scan_complete_tx.clone()).await?;

                    return Ok(Some("scan_in_progress".to_string()));
                }
                MainMenuOptions::KnownNetworks => {
                    self.current_menu = CurrentMenu::KnownNetworksMenu;
                    loop {
                        let selected_network = {
                            if let Some(station_arc) = self.adapter.device.station.as_ref() {
                                let station_guard = station_arc.read().await;
                                let known_networks = station_guard
                                    .known_networks
                                    .iter()
                                    .filter_map(|(network, signal_strength)| {
                                        network.known_network.as_ref().map(|known_network| {
                                            (known_network.clone(), *signal_strength)
                                        })
                                    })
                                    .collect::<Vec<(KnownNetwork, i16)>>();
                                drop(station_guard);

                                menu.show_known_networks_menu(
                                    menu_command,
                                    &known_networks[..],
                                    icon_type,
                                    spaces,
                                )
                                .await?
                            } else {
                                None
                            }
                        };

                        if let Some(selected_network) = selected_network {
                            self.handle_known_network_options(
                                menu,
                                menu_command,
                                &selected_network,
                                icon_type,
                                spaces,
                            )
                            .await?;

                            continue;
                        }

                        break;
                    }

                    return Ok(Some("handled_known_networks".to_string()));
                }
                MainMenuOptions::Network(output) => {
                    if let Some(ssid) = self
                        .handle_network_selection(menu, menu_command, &output, icon_type, spaces)
                        .await?
                    {
                        return Ok(Some(ssid));
                    }
                }
                MainMenuOptions::Settings => {
                    self.current_menu = CurrentMenu::SettingsMenu;
                    self.handle_settings(menu, menu_command, icon_type, spaces, scan_complete_tx)
                        .await?;
                    return Ok(Some("handled_settings".to_string()));
                }
                _ => {
                    self.log_sender
                        .send(t!("notifications.app.menu_exit").to_string())
                        .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));
                    self.running = false;
                    return Ok(None);
                }
            }
        } else {
            self.log_sender
                .send(t!("notifications.app.no_network_selected").to_string())
                .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));
            self.running = false;
            return Ok(None);
        }

        Ok(None)
    }

    async fn handle_scan(&mut self, scan_complete_tx: UnboundedSender<()>) -> Result<()> {

        if let Some(station_arc) = self.adapter.device.station.as_ref() {
            let station_guard = station_arc.write().await;

            station_guard
                .scan(
                    self.log_sender.clone(),
                    Arc::clone(&self.notification_manager),
                    scan_complete_tx,
                )
                .await?;
        }
        Ok(())
    }

    async fn handle_device_off(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
        scan_complete_tx: UnboundedSender<()>,
    ) -> Result<()> {
        if let Ok(Some(option)) = menu
            .prompt_enable_adapter(menu_command, icon_type, spaces)
            .await
        {
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

                    if let Some(station_arc) = self.adapter.device.station.as_ref() {
                        let mut station_guard = station_arc.write().await;

                        station_guard
                            .scan(
                                self.log_sender.clone(),
                                Arc::clone(&self.notification_manager),
                                scan_complete_tx.clone(),
                            )
                            .await?;
                        station_guard.refresh(self.log_sender.clone()).await?;
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

            if let Some(station_arc) = self.adapter.device.station.as_ref() {
                let mut station_guard = station_arc.write().await;
                station_guard.refresh(self.log_sender.clone()).await?;
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
        scan_complete_tx: UnboundedSender<()>,
    ) -> Result<()> {
        loop {
            if let Some(ap_arc) = self.adapter.device.access_point.as_ref() {
                let ap_menu_option;
                {
                    let ap_guard = ap_arc.write().await;

                    if let Ok(Some(output)) = menu
                        .show_ap_menu(menu_command, &*ap_guard, icon_type, spaces)
                        .await
                    {
                        let cleaned_output = menu.clean_menu_output(&output, icon_type);

                        if let Some(option) = ApMenuOptions::from_str(&cleaned_output) {
                            ap_menu_option = Some(option);
                        } else {
                            ap_menu_option = None;
                        }
                    } else {
                        ap_menu_option = None;
                    }
                }

                if let Some(ap_menu_option) = ap_menu_option {
                    match ap_menu_option {
                        ApMenuOptions::StartAp => {
                            self.start_ap(menu, menu_command, icon_type).await?;
                        }
                        ApMenuOptions::StopAp => self.stop_ap().await?,
                        ApMenuOptions::SetSsid => {
                            if let Ok(Some(ssid)) = menu.prompt_ssid(menu_command, icon_type).await
                            {
                                let mut ap_guard = ap_arc.write().await;
                                ap_guard.set_ssid(ssid.clone());
                                self.log_sender
                                    .send(format!("SSID défini sur {}", ssid))
                                    .unwrap_or_else(|err| {
                                        println!("Échec de l'envoi du message : {}", err)
                                    });
                            }
                        }
                        ApMenuOptions::SetPassword => {
                            if let Ok(Some(password)) =
                                menu.prompt_password(menu_command, icon_type).await
                            {
                                let mut ap_guard = ap_arc.write().await;
                                ap_guard.set_psk(password.clone());
                                self.log_sender
                                    .send("Mot de passe défini".to_string())
                                    .unwrap_or_else(|err| {
                                        println!("Échec de l'envoi du message : {}", err)
                                    });
                            }
                        }
                        ApMenuOptions::Settings => {
                            self.handle_settings(
                                menu,
                                menu_command,
                                icon_type,
                                spaces,
                                scan_complete_tx.clone(),
                            )
                            .await?;
                        }
                    }
                } else {
                    self.running = false;
                    break;
                }
            } else {
                self.log_sender
                    .send("Aucun point d'accès disponible".to_string())
                    .unwrap_or_else(|err| println!("Échec de l'envoi du message : {}", err));
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

    async fn handle_network_selection(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        output: &str,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        if let Some(station_arc) = self.adapter.device.station.as_ref() {
            let mut station_guard = station_arc.write().await;

            let networks = station_guard
                .new_networks
                .iter()
                .chain(station_guard.known_networks.iter());

            if let Some((network, _)) =
                menu.select_network(networks, output.to_string(), icon_type, spaces)
            {
                if station_guard
                    .connected_network
                    .as_ref()
                    .map_or(false, |cn| cn.name == network.name)
                {
                    station_guard
                        .disconnect(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                    station_guard.refresh(self.log_sender.clone()).await?;
                    return Ok(None);
                }

                if network.known_network.is_some() {
                    self.log_sender
                        .send(format!("Connecting to known network: {}", network.name))
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    network
                        .connect(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                    station_guard.refresh(self.log_sender.clone()).await?;
                    return Ok(Some(network.name.clone()));
                } else {
                    self.log_sender
                        .send(format!("Connecting to new network: {}", network.name))
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    if let Ok(Some(passphrase)) = menu
                        .prompt_passphrase(menu_command, &network.name, icon_type)
                        .await
                    {
                        self.agent_manager.send_passkey(passphrase)?;
                    } else {
                        self.agent_manager.cancel_auth()?;
                        return Ok(None);
                    }

                    network
                        .connect(self.log_sender.clone(), self.notification_manager.clone())
                        .await?;
                    station_guard.refresh(self.log_sender.clone()).await?;
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
        scan_complete_tx: UnboundedSender<()>,
    ) -> Result<()> {
        if let Some(option) = menu
            .show_settings_menu(menu_command, &self.current_mode, icon_type, spaces)
            .await?
        {
            match option {
                SettingsMenuOptions::DisableAdapter => {
                    self.disable_adapter(menu, menu_command, icon_type, spaces, scan_complete_tx)
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
            _ => Mode::Station,
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
        scan_complete_tx: UnboundedSender<()>,
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

        self.handle_device_off(
            menu,
            menu_command,
            icon_type,
            spaces,
            scan_complete_tx.clone(),
        )
        .await?;

        Ok(())
    }

    async fn start_ap(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
    ) -> Result<()> {
        if let Some(ap_arc) = self.adapter.device.access_point.as_ref() {
            let ssid;
            let psk;

            {
                let mut ap_guard = ap_arc.write().await;

                if ap_guard.has_started {
                    self.log_sender
                        .send("Access Point is already started".to_string())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                    return Ok(());
                }

                ssid = if ap_guard.ssid.is_empty() {
                    menu.prompt_ssid(menu_command, icon_type)
                        .await
                        .unwrap_or_else(|_| Some("MySSID".to_string()))
                } else {
                    Some(ap_guard.ssid.clone())
                };

                psk = if ap_guard.psk.is_empty() {
                    menu.prompt_password(menu_command, icon_type)
                        .await
                        .unwrap_or_else(|_| Some("MyPassword".to_string()))
                } else {
                    Some(ap_guard.psk.clone())
                };

                ap_guard.set_ssid(ssid.clone().unwrap_or_else(|| "MySSID".to_string()));
                ap_guard.set_psk(psk.clone().unwrap_or_else(|| "MyPassword".to_string()));
            }

            self.adapter.refresh(self.log_sender.clone()).await?;
            self.log_sender
                .send("Access Point started successfully".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            self.notification_manager.send_notification(
                None,
                Some("Access Point started successfully".to_string()),
                None,
                None,
            );
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
        if let Some(ap_arc) = &self.adapter.device.access_point {
            {
                let ap_guard = ap_arc.write().await;
                ap_guard.stop().await?;
            }

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
