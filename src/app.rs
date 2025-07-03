use crate::{
    icons::Icons,
    iw::{adapter::Adapter, agent::AgentManager, known_network::KnownNetwork, network::Network},
    menu::{
        AdapterMenuOptions, ApMenuOptions, KnownNetworkOptions, MainMenuOptions, Menu,
        SettingsMenuOptions,
    },
    notification::NotificationManager,
};
use anyhow::{anyhow, Context, Error, Result};
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
        icons: Arc<Icons>,
    ) -> Result<Self> {
        let agent_manager = AgentManager::new().await?;
        let session = agent_manager.session();
        let adapter = Adapter::new(session.clone(), log_sender.clone()).await?;
        let current_mode = adapter.device.mode.clone();

        let notification_manager = Arc::new(NotificationManager::new(icons.clone()));

        if !adapter.device.is_powered {
            adapter
                .device
                .power_on()
                .await
                .with_context(|| "Failed to power on the adapter during initialization")?;
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
        let adapter = Adapter::new(session.clone(), log_sender.clone())
            .await
            .with_context(|| "Failed to create a new adapter during reset")?;

        adapter
            .device
            .set_mode(mode.clone())
            .await
            .with_context(|| format!("Failed to set mode to {mode:?} during reset"))?;

        self.adapter = adapter;
        self.session = session;
        self.current_mode = mode;

        try_send_log!(
            self.log_sender,
            format!("App state reset with mode: {:?}", self.current_mode)
        );

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
            if self.running {
                self.adapter
                    .refresh()
                    .await
                    .with_context(|| "Failed to refresh adapter state after power-on")?;
            } else {
                return Ok(None);
            }
        }

        while self.running {
            self.adapter.refresh().await?;

            match self.adapter.device.mode {
                Mode::Station => {
                    self.run_station_mode(menu, menu_command, icon_type, spaces)
                        .await?;
                }
                Mode::Ap => {
                    self.run_ap_mode(menu, menu_command, icon_type, spaces)
                        .await?;
                }
                _ => {
                    try_send_log!(
                        self.log_sender,
                        t!("notifications.app.unknown_mode").to_string()
                    );
                    self.running = false;
                }
            }
        }

        Ok(None)
    }

    async fn run_ap_mode(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        let access_point = match self.adapter.device.access_point.as_mut() {
            Some(ap) => ap,
            None => {
                try_send_log!(
                    self.log_sender,
                    t!("notifications.app.no_access_point_available").to_string()
                );
                self.running = false;
                return Ok(());
            }
        };

        match menu
            .show_ap_menu(menu_command, access_point, icon_type, spaces)
            .await?
        {
            Some(ap_menu_option) => {
                self.handle_ap_options(ap_menu_option, menu, menu_command, icon_type, spaces)
                    .await?;
            }
            None => {
                try_send_log!(
                    self.log_sender,
                    t!("notifications.app.ap_menu_exited").to_string()
                );
                self.running = false;
            }
        }

        Ok(())
    }

    async fn wait_for_scan_completion(station: &mut crate::iw::station::Station) -> Result<()> {
        const SCAN_TIMEOUT_SECS: u64 = 30;
        const SCAN_POLL_INTERVAL_MS: u64 = 250;

        let result = tokio::time::timeout(Duration::from_secs(SCAN_TIMEOUT_SECS), async {
            while station.is_scanning {
                sleep(Duration::from_millis(SCAN_POLL_INTERVAL_MS)).await;
                station.refresh().await?;
            }
            Ok::<(), Error>(())
        })
        .await;

        match result {
            Ok(inner_result) => inner_result,
            Err(_) => Err(anyhow!("Station scan timeout exceeded during run loop")),
        }
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
                self.handle_settings_menu(menu, menu_command, icon_type, spaces)
                    .await?;
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
                        try_send_log!(self.log_sender, "SSID or Password not set".to_string());
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
                ApMenuOptions::StopAp => {
                    self.perform_ap_stop().await?;
                }
                ApMenuOptions::SetSsid => {
                    if let Some(ssid) = menu.prompt_ap_ssid(menu_command, icon_type) {
                        ap.set_ssid(ssid.clone());
                        try_send_log!(self.log_sender, format!("SSID set to {ssid}"));
                    }
                }
                ApMenuOptions::SetPassword => {
                    if let Some(password) = menu.prompt_ap_passphrase(menu_command, icon_type) {
                        ap.set_psk(password.clone());
                        try_send_log!(self.log_sender, "Password set".to_string());
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
    ) -> Result<bool> {
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
            .show_known_network_options(
                menu_command,
                icon_type,
                spaces,
                available_options,
                &known_network.name,
            )
            .await?
        {
            match option {
                KnownNetworkOptions::DisableAutoconnect => {
                    self.perform_toggle_autoconnect(known_network, false)
                        .await?;
                    Ok(true)
                }
                KnownNetworkOptions::EnableAutoconnect => {
                    self.perform_toggle_autoconnect(known_network, true).await?;
                    Ok(true)
                }
                KnownNetworkOptions::ForgetNetwork => {
                    self.perform_forget_network(known_network).await?;
                    Ok(false)
                }
                KnownNetworkOptions::Disconnect => {
                    if is_connected {
                        self.perform_network_disconnection().await?;
                    }
                    Ok(true)
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
                    Ok(true)
                }
            }
        } else {
            try_send_log!(
                self.log_sender,
                format!("Exited network menu for {}", known_network.name)
            );
            Ok(false)
        }
    }

    async fn handle_settings_menu(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        let mut stay_in_settings_menu = true;

        while stay_in_settings_menu {
            self.adapter.refresh().await?;

            if let Some(option) = menu
                .show_settings_menu(menu_command, &self.current_mode, icon_type, spaces)
                .await?
            {
                let should_stay = self
                    .handle_settings_options(option, menu, menu_command, icon_type, spaces)
                    .await?;

                if !should_stay {
                    stay_in_settings_menu = false;
                }
            } else {
                try_send_log!(self.log_sender, "Exited settings menu".to_string());
                stay_in_settings_menu = false;
            }
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
    ) -> Result<bool> {
        match option {
            SettingsMenuOptions::DisableAdapter => {
                self.perform_adapter_disable(menu, menu_command, icon_type, spaces)
                    .await?;
                Ok(false)
            }
            SettingsMenuOptions::SwitchMode => {
                self.perform_mode_switch(menu).await?;
                self.reset_mode = true;
                self.running = false;
                Ok(false)
            }
        }
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
                    self.reset(self.current_mode.clone(), self.log_sender.clone())
                        .await?;
                    try_send_log!(
                        self.log_sender,
                        t!("notifications.app.adapter_enabled").to_string()
                    );
                    try_send_notification!(
                        self.notification_manager,
                        None,
                        Some(t!("notifications.app.adapter_enabled").to_string()),
                        Some("network_wireless"),
                        None
                    );
                }
            }
        } else {
            try_send_log!(
                self.log_sender,
                t!("notifications.app.adapter_menu_exited").to_string()
            );
            self.running = false;
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
        let station = self
            .adapter
            .device
            .station
            .as_mut()
            .ok_or_else(|| anyhow!("No station available for network selection"))?;

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
                    .is_some_and(|cn| cn.name == network.name);

                self.handle_network_menu(
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

        Ok(None)
    }

    async fn handle_network_menu(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        known_network: &KnownNetwork,
        icon_type: &str,
        spaces: usize,
        is_connected: bool,
    ) -> Result<()> {
        let mut stay_in_network_menu = true;
        let mut network_clone = known_network.clone();
        let mut current_is_connected = is_connected;

        while stay_in_network_menu {
            if let Some(station) = self.adapter.device.station.as_mut() {
                station.refresh().await?;

                current_is_connected = station
                    .connected_network
                    .as_ref()
                    .is_some_and(|cn| cn.name == network_clone.name);

                if let Some((updated_network, _)) = station
                    .known_networks
                    .iter()
                    .find(|(net, _)| net.name == network_clone.name)
                {
                    if let Some(ref updated_known_network) = updated_network.known_network {
                        network_clone = updated_known_network.clone();
                    } else {
                        try_send_log!(
                            self.log_sender,
                            format!("Network {} is no longer available", network_clone.name)
                        );
                        break;
                    }
                } else {
                    try_send_log!(
                        self.log_sender,
                        format!("Network {} is no longer available", network_clone.name)
                    );
                    break;
                }
            }

            let should_stay = self
                .handle_known_network_options(
                    menu,
                    menu_command,
                    &network_clone,
                    icon_type,
                    spaces,
                    current_is_connected,
                )
                .await?;

            if !should_stay {
                stay_in_network_menu = false;
            }

            if let Some(station) = self.adapter.device.station.as_mut() {
                station.refresh().await?;
            }
        }

        Ok(())
    }

    async fn perform_known_network_connection(
        &mut self,
        network: &Network,
    ) -> Result<Option<String>> {
        let station = self
            .adapter
            .device
            .station
            .as_mut()
            .ok_or_else(|| anyhow!("No station available for known network connection"))?;

        try_send_log!(
            self.log_sender,
            format!("Connecting to known network: {}", network.name)
        );

        network
            .connect()
            .await
            .with_context(|| format!("Failed to connect to known network: {}", network.name))?;

        let msg = t!(
            "notifications.network.connected",
            network_name = network.name
        );
        try_send_log!(self.log_sender, msg.to_string());
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some("connected"),
            None
        );

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
        let station = self
            .adapter
            .device
            .station
            .as_mut()
            .ok_or_else(|| anyhow!("No station available for new network connection"))?;

        try_send_log!(
            self.log_sender,
            format!("Connecting to new network: {}", network.name)
        );

        if let Some(passphrase) =
            menu.prompt_station_passphrase(menu_command, &network.name, icon_type)
        {
            self.agent_manager.send_passkey(passphrase)?;
        } else {
            self.agent_manager.cancel_auth()?;
            return Ok(None);
        }

        network
            .connect()
            .await
            .with_context(|| format!("Failed to connect to new network: {}", network.name))?;

        let msg = t!(
            "notifications.network.connected",
            network_name = network.name
        );
        try_send_log!(self.log_sender, msg.to_string());
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some("connected"),
            None
        );

        station.refresh().await?;
        Ok(Some(network.name.clone()))
    }

    pub async fn perform_network_disconnection(&mut self) -> Result<()> {
        let station = self
            .adapter
            .device
            .station
            .as_mut()
            .ok_or_else(|| anyhow!("No station available for disconnection"))?;

        let connected_network_name = station
            .connected_network
            .as_ref()
            .ok_or_else(|| anyhow!("No network is currently connected"))?
            .name
            .clone();

        try_send_log!(
            self.log_sender,
            format!("Disconnecting from network: {connected_network_name}")
        );

        station.disconnect().await?;

        let msg = t!(
            "notifications.station.disconnected_from_network",
            network_name = connected_network_name
        );

        try_send_log!(self.log_sender, msg.to_string());
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some("disconnected"),
            None
        );

        station.refresh().await?;
        Ok(())
    }

    async fn perform_network_scan(&mut self) -> Result<()> {
        if let Some(station) = self.adapter.device.station.as_mut() {
            if station.is_scanning {
                let msg = t!("notifications.station.scan_already_in_progress");
                try_send_log!(self.log_sender, msg.to_string());
                try_send_notification!(
                    self.notification_manager,
                    None,
                    Some(msg.to_string()),
                    Some("scan"),
                    None
                );
                return Ok(());
            }

            station.scan().await?;

            let notification_id = try_send_notification_with_id!(
                self.notification_manager,
                None,
                Some(t!("notifications.station.scan_in_progress").to_string()),
                Some("scan_in_progress"),
                Some(Timeout::Never)
            );

            while station.is_scanning {
                sleep(Duration::from_millis(500)).await;
                station.refresh().await?;
            }

            station.refresh().await?;

            if let Some(id) = notification_id {
                self.notification_manager.close_notification(id)?;
            }

            let msg = t!("notifications.station.scan_completed");
            self.log_sender.send(msg.to_string()).unwrap_or_default();
            try_send_notification!(
                self.notification_manager,
                None,
                Some(msg.to_string()),
                Some("ok"),
                None
            );
        } else {
            return Err(anyhow!("No station available for scanning"));
        }

        Ok(())
    }

    async fn perform_forget_network(&self, known_network: &KnownNetwork) -> Result<()> {
        known_network
            .forget()
            .await
            .with_context(|| format!("Failed to forget network {}", known_network.name))?;

        let msg = t!(
            "notifications.known_networks.forget_network",
            network_name = known_network.name
        );
        try_send_log!(self.log_sender, msg.clone().into_owned());
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some("forget_network"),
            None
        );

        Ok(())
    }

    async fn perform_toggle_autoconnect(
        &self,
        known_network: &KnownNetwork,
        enable: bool,
    ) -> Result<()> {
        known_network
            .toggle_autoconnect(enable)
            .await
            .with_context(|| {
                format!(
                    "Failed to {} auto-connect for network {}",
                    if enable { "enable" } else { "disable" },
                    known_network.name
                )
            })?;

        let (msg, icon) = if enable {
            (
                t!(
                    "notifications.known_networks.enable_autoconnect",
                    network_name = known_network.name
                ),
                "enable_autoconnect",
            )
        } else {
            (
                t!(
                    "notifications.known_networks.disable_autoconnect",
                    network_name = known_network.name
                ),
                "disable_autoconnect",
            )
        };

        try_send_log!(self.log_sender, msg.clone().into_owned());
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some(icon),
            None
        );

        Ok(())
    }

    async fn perform_mode_switch(&mut self, menu: &Menu) -> Result<()> {
        let new_mode = match self.current_mode {
            Mode::Station => Mode::Ap,
            Mode::Ap => Mode::Station,
            _ => {
                let msg = t!("notifications.app.unknown_mode").to_string();
                try_send_log!(self.log_sender, msg.clone());
                return Err(anyhow!("Unsupported mode"));
            }
        };

        self.reset(new_mode.clone(), self.log_sender.clone())
            .await
            .context("Failed to reset application state during mode switch")?;

        let mode_text = menu.get_mode_text(&new_mode);
        let msg = t!("notifications.device.switched_mode", mode = mode_text).to_string();

        try_send_log!(self.log_sender, msg.clone());

        let icon = match new_mode {
            Mode::Ap => "access_point",
            Mode::Station => "station",
            _ => "unknown",
        };

        try_send_notification!(self.notification_manager, None, Some(msg), Some(icon), None);

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

        let msg = t!("notifications.app.adapter_disabled").to_string();
        try_send_log!(self.log_sender, msg.clone());
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg),
            Some("disable_adapter"),
            None
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
                let msg = "Access point is already started".to_string();
                try_send_log!(self.log_sender, msg);
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

            ap.start().await?;

            let msg = t!("notifications.device.access_point_started").to_string();
            try_send_log!(
                self.log_sender,
                "Access Point started successfully".to_string()
            );
            try_send_notification!(
                self.notification_manager,
                None,
                Some(msg),
                Some("access_point"),
                None
            );

            self.adapter.refresh().await?;
        } else {
            let msg = "No access point available to start".to_string();
            try_send_log!(self.log_sender, msg.clone());
            try_send_notification!(
                self.notification_manager,
                None,
                Some(t!("notifications.device.no_access_point_available").to_string()),
                Some("error"),
                None
            );
        }

        Ok(())
    }

    async fn perform_ap_stop(&mut self) -> Result<()> {
        if let Some(ap) = &self.adapter.device.access_point {
            ap.stop().await?;
            self.adapter.refresh().await?;

            let msg = "Access point stopped".to_string();
            try_send_log!(self.log_sender, msg.clone());
            try_send_notification!(
                self.notification_manager,
                None,
                Some(t!("notifications.device.access_point_stopped").to_string()),
                Some("access_point"),
                None
            );
        } else {
            return Err(anyhow!("No access point available to stop"));
        }

        Ok(())
    }

    async fn run_station_mode(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        let station = match self.adapter.device.station.as_mut() {
            Some(station) => station,
            None => {
                try_send_log!(
                    self.log_sender,
                    t!("notifications.app.no_station_available").to_string()
                );
                self.running = false;
                return Ok(());
            }
        };

        if station.is_scanning {
            Self::wait_for_scan_completion(station).await?;
        }

        match menu
            .show_main_menu(menu_command, station, icon_type, spaces)
            .await?
        {
            Some(main_menu_option) => {
                self.handle_main_options(menu, menu_command, icon_type, spaces, main_menu_option)
                    .await?;
            }
            None => {
                try_send_log!(
                    self.log_sender,
                    t!("notifications.app.main_menu_exited").to_string()
                );
                self.running = false;
            }
        }

        Ok(())
    }
}
