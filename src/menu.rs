use anyhow::Result;
use clap::ArgEnum;
use std::{
    collections::HashMap,
    io::Write,
    process::{Command, Stdio},
};

use crate::iw::{
    access_point::AccessPoint, adapter::Adapter, known_network::KnownNetwork, network::Network,
    station::Station,
};

#[derive(Debug, Clone, ArgEnum)]
pub enum MenuType {
    Fuzzel,
    Wofi,
    Rofi,
    Dmenu,
    Custom,
}

#[derive(Debug, Clone)]
pub enum MainMenuOptions {
    Scan,
    KnownNetworks,
    Settings,
    Network(String),
}

impl MainMenuOptions {
    pub fn from_str(option: &str) -> Option<Self> {
        match option {
            "Scan" => Some(MainMenuOptions::Scan),
            "Known Networks" => Some(MainMenuOptions::KnownNetworks),
            "Settings" => Some(MainMenuOptions::Settings),
            other => Some(MainMenuOptions::Network(other.to_string())),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            MainMenuOptions::Scan => "Scan",
            MainMenuOptions::KnownNetworks => "Known Networks",
            MainMenuOptions::Settings => "Settings",
            MainMenuOptions::Network(_) => "Network",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KnownNetworkOptions {
    DisableAutoconnect,
    EnableAutoconnect,
    ForgetNetwork,
}

impl KnownNetworkOptions {
    pub fn from_str(option: &str) -> Option<Self> {
        match option {
            "Disable Autoconnect" => Some(KnownNetworkOptions::DisableAutoconnect),
            "Enable Autoconnect" => Some(KnownNetworkOptions::EnableAutoconnect),
            "Forget Network" => Some(KnownNetworkOptions::ForgetNetwork),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            KnownNetworkOptions::DisableAutoconnect => "Disable Autoconnect",
            KnownNetworkOptions::EnableAutoconnect => "Enable Autoconnect",
            KnownNetworkOptions::ForgetNetwork => "Forget Network",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsMenuOptions {
    DisableAdapter,
    ChangeMode,
}

impl SettingsMenuOptions {
    pub fn from_str(option: &str) -> Option<Self> {
        match option {
            "Disable Adapter" => Some(SettingsMenuOptions::DisableAdapter),
            "Change Mode" => Some(SettingsMenuOptions::ChangeMode),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            SettingsMenuOptions::DisableAdapter => "Disable Adapter",
            SettingsMenuOptions::ChangeMode => "Change Mode",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "disable_adapter" => Some(SettingsMenuOptions::DisableAdapter),
            "change_mode" => Some(SettingsMenuOptions::ChangeMode),
            _ => None,
        }
    }

    pub fn to_id(&self) -> &'static str {
        match self {
            SettingsMenuOptions::DisableAdapter => "disable_adapter",
            SettingsMenuOptions::ChangeMode => "change_mode",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChangeModeMenuOptions {
    Station,
    Ap,
}

impl ChangeModeMenuOptions {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "station" => Some(ChangeModeMenuOptions::Station),
            "ap" => Some(ChangeModeMenuOptions::Ap),
            _ => None,
        }
    }

    pub fn to_id(&self) -> &'static str {
        match self {
            ChangeModeMenuOptions::Station => "station",
            ChangeModeMenuOptions::Ap => "ap",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ApMenuOptions {
    StartAp,
    StopAp,
    SetSsid,
    SetPassword,
    ChangeMode,
}

impl ApMenuOptions {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "start_ap" => Some(ApMenuOptions::StartAp),
            "stop_ap" => Some(ApMenuOptions::StopAp),
            "set_ssid" => Some(ApMenuOptions::SetSsid),
            "set_password" => Some(ApMenuOptions::SetPassword),
            "change_mode" => Some(ApMenuOptions::ChangeMode),
            _ => None,
        }
    }

    pub fn to_id(&self) -> &'static str {
        match self {
            ApMenuOptions::StartAp => "start_ap",
            ApMenuOptions::StopAp => "stop_ap",
            ApMenuOptions::SetSsid => "set_ssid",
            ApMenuOptions::SetPassword => "set_password",
            ApMenuOptions::ChangeMode => "change_mode",
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            ApMenuOptions::StartAp => "Start AP",
            ApMenuOptions::StopAp => "Stop AP",
            ApMenuOptions::SetSsid => "Set SSID",
            ApMenuOptions::SetPassword => "Set Password",
            ApMenuOptions::ChangeMode => "Change Mode",
        }
    }
}

#[derive(Clone)]
pub struct Menu {
    pub menu_type: MenuType,
    pub icons: Icons,
}

#[derive(Clone)]
pub struct Icons {
    font_icons: HashMap<&'static str, char>,
    xdg_icons: HashMap<&'static str, &'static str>,
}

impl Icons {
    pub fn new() -> Self {
        let mut font_icons = HashMap::new();
        let mut xdg_icons = HashMap::new();

        font_icons.insert("signal_weak_open", '\u{f16cb}');
        font_icons.insert("signal_weak_secure", '\u{f0921}');
        font_icons.insert("signal_ok_open", '\u{f16cc}');
        font_icons.insert("signal_ok_secure", '\u{f0924}');
        font_icons.insert("signal_good_open", '\u{f16cd}');
        font_icons.insert("signal_good_secure", '\u{f0927}');
        font_icons.insert("signal_excellent_open", '\u{f16ce}');
        font_icons.insert("signal_excellent_secure", '\u{f092a}');
        font_icons.insert("connected", '\u{f0133}');
        font_icons.insert("known_network", '\u{f16bd}');
        font_icons.insert("scan", '\u{f46a}');
        font_icons.insert("known_networks", '\u{f0134}');
        font_icons.insert("settings", '\u{f0493}');
        font_icons.insert("disable_adapter", '\u{f092d}');
        font_icons.insert("power_on_device", '\u{f0425}');
        font_icons.insert("change_mode", '\u{f0fe2}');
        font_icons.insert("start_ap", '\u{f040d}');
        font_icons.insert("stop_ap", '\u{f0667}');
        font_icons.insert("set_ssid", '\u{f08d5}');
        font_icons.insert("set_password", '\u{f0bc5}');
        font_icons.insert("enable_autoconnect", '\u{f0547}');
        font_icons.insert("disable_autoconnect", '\u{f0547}');
        font_icons.insert("forget_network", '\u{f01b4}');
        font_icons.insert("station", '\u{f05a9}');
        font_icons.insert("access_point", '\u{f0003}');

        xdg_icons.insert("signal_weak_open", "network-wireless-signal-weak-symbolic");
        xdg_icons.insert("signal_ok_open", "network-wireless-signal-ok-symbolic");
        xdg_icons.insert("signal_good_open", "network-wireless-signal-good-symbolic");
        xdg_icons.insert(
            "signal_excellent_open",
            "network-wireless-signal-excellent-symbolic",
        );
        xdg_icons.insert("signal_weak_secure", "network-wireless-signal-weak-secure-symbolic");
        xdg_icons.insert("signal_ok_secure", "network-wireless-signal-ok-secure-symbolic");
        xdg_icons.insert("signal_good_secure", "network-wireless-signal-good--securesymbolic");
        xdg_icons.insert(
            "signal_excellent_secure",
            "network-wireless-signal-excellent-secure-symbolic",
        );
        xdg_icons.insert("scan", "view-refresh-symbolic");
        xdg_icons.insert("known_networks", "app-installed-symbolic");
        xdg_icons.insert("settings", "preferences-system-symbolic");
        xdg_icons.insert(
            "disable_adapter",
            "network-wireless-hardware-disabled-symbolic",
        );
        xdg_icons.insert("power_on_device", "system-shutdown-symbolic");
        xdg_icons.insert("change_mode", "system-switch-user-symbolic");
        xdg_icons.insert("start_ap", "media-playback-start-symbolic");
        xdg_icons.insert("stop_ap", "media-playback-stop-symbolic");
        xdg_icons.insert("set_ssid", "edit-paste-symbolic");
        xdg_icons.insert("set_password", "safety-symbolic");
        xdg_icons.insert("enable_autoconnect", "media-playlist-repeat-symbolic");
        xdg_icons.insert("disable_autoconnect", "media-playlist-repeat-symbolic");
        xdg_icons.insert("forget_network", "close-symbolic");
        xdg_icons.insert("connected", "network-wireless-connected-symbolic");
        xdg_icons.insert("known_network", "network-wireless-connected-symbolic");
        xdg_icons.insert("station", "network-wireless-symbolic");
        xdg_icons.insert("access_point", "network-cellular-symbolic");

        Icons {
            font_icons,
            xdg_icons,
        }
    }

    pub fn get_icon(&self, key: &str, icon_type: &str) -> String {
        match icon_type {
            "font" => self
                .font_icons
                .get(key)
                .map_or(String::new(), |&icon| icon.to_string()),
            "xdg" => self
                .xdg_icons
                .get(key)
                .map_or(String::new(), |&icon| icon.to_string()),
            _ => String::new(),
        }
    }

    pub fn get_icon_char(&self, key: &str) -> Option<char> {
        self.font_icons.get(key).copied()
    }

    pub fn get_icon_text(
        &self,
        items: Vec<(&str, &str)>,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        items
            .into_iter()
            .map(|(icon_key, text)| {
                let icon = self.get_icon(icon_key, icon_type);
                match icon_type {
                    "font" => format!("{}{}{}", icon, " ".repeat(spaces), text),
                    "xdg" => format!("{}\0icon\x1f{}", text, icon),
                    _ => text.to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn get_signal_icon(
        &self,
        signal_strength: i16,
        network_type: &str,
        icon_type: &str,
    ) -> String {
        let icon_key = match signal_strength {
            -10000..=-7500 => match network_type {
                "open" => "signal_weak_open",
                _ => "signal_weak_secure",
            },
            -7499..=-5000 => match network_type {
                "open" => "signal_ok_open",
                _ => "signal_ok_secure",
            },
            -4999..=-2500 => match network_type {
                "open" => "signal_good_open",
                _ => "signal_good_secure",
            },
            _ => match network_type {
                "open" => "signal_excellent_open",
                _ => "signal_excellent_secure",
            },
        };

        self.get_icon(icon_key, icon_type)
    }

    pub fn get_connected_icon(&self) -> Option<char> {
        self.get_icon_char("connected")
    }

    pub fn format_with_spacing(icon: char, spaces: usize, before: bool) -> String {
        if before {
            format!("{}{}", " ".repeat(spaces), icon)
        } else {
            format!("{}{}", icon, " ".repeat(spaces))
        }
    }

    pub fn format_display_with_icon(
        &self,
        name: &str,
        icon: &str,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        if icon_type == "xdg" {
            format!("{}\0icon\x1f{}", name, icon)
        } else {
            format!("{}{}{}", icon, " ".repeat(spaces), name)
        }
    }

    pub fn format_network_display(
        &self,
        network: &Network,
        signal_strength: i16,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        let signal_icon = self.get_signal_icon(signal_strength, &network.network_type, icon_type);
        let mut display = network.name.clone();

        if network.is_connected {
            if icon_type == "xdg" {
                display = format!("{} \u{2705}", display);
            } else if icon_type == "font" {
                if let Some(connected_icon) = self.get_connected_icon() {
                    display.push_str(&Icons::format_with_spacing(connected_icon, spaces, true));
                }
            }
        }

        self.format_display_with_icon(&display, &signal_icon, icon_type, spaces)
    }

    pub fn format_known_network_display(
        &self,
        known_network: &KnownNetwork,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        let connected_icon = self.get_icon("connected", icon_type);

        if icon_type == "font" {
            if let Some(icon) = self.get_connected_icon() {
                format!(
                    "{}{}",
                    Self::format_with_spacing(icon, spaces, false),
                    known_network.name
                )
            } else {
                known_network.name.clone()
            }
        } else {
            format!("{}\0icon\x1f{}", known_network.name, connected_icon)
        }
    }
}

impl Menu {
    pub fn new(menu_type: MenuType) -> Self {
        Self {
            menu_type,
            icons: Icons::new(),
        }
    }

    pub fn run_menu_command(
        &self,
        menu_command: &Option<String>,
        input: Option<&str>,
        icon_type: &str,
    ) -> Option<String> {
        let output = match self.menu_type {
            MenuType::Fuzzel => {
                let mut command = Command::new("fuzzel");
                command.arg("-d");

                if icon_type == "font" {
                    command.arg("-I");
                }


                let mut child = command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .ok()?;

                if let Some(input_data) = input {
                        child
                            .stdin
                            .as_mut()
                            .unwrap()
                        .write_all(input_data.as_bytes())
                            .unwrap();
                }

                let output = child.wait_with_output().ok()?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            MenuType::Wofi => {
                let mut command = Command::new("wofi");
                command.arg("-d").arg("-i");

                if icon_type == "xdg" {
                    command.arg("-I").arg("-m").arg("-q");
                }

                let mut child = command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .ok()?;

                if let Some(input_data) = input {
                        child
                            .stdin
                            .as_mut()
                            .unwrap()
                        .write_all(input_data.as_bytes())
                            .unwrap();
                }

                let output = child.wait_with_output().ok()?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            MenuType::Rofi => {
                let mut command = Command::new("rofi");
                command.arg("-m").arg("-1").arg("-dmenu");

                if icon_type == "xdg" {
                    command.arg("-show-icons");
                }


                let mut child = command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .ok()?;

                if let Some(input_data) = input {
                        child
                            .stdin
                            .as_mut()
                            .unwrap()
                        .write_all(input_data.as_bytes())
                            .unwrap();
                }

                let output = child.wait_with_output().ok()?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            MenuType::Dmenu => {
                let mut command = Command::new("dmenu");

                let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                    .ok()?;

                if let Some(input_data) = input {
                    child
                        .stdin
                        .as_mut()
                        .unwrap()
                        .write_all(input_data.as_bytes())
                        .unwrap();
                }

                let output = child.wait_with_output().ok()?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            MenuType::Custom => {
                if let Some(cmd) = menu_command {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    let (cmd, args) = parts.split_first().unwrap();
                    let mut command = Command::new(cmd);
                    command.args(args);

                    let mut child = command
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .ok()?;

                    if let Some(input_data) = input {
                            child
                                .stdin
                                .as_mut()
                                .unwrap()
                            .write_all(input_data.as_bytes())
                                .unwrap();
                    }

                    let output = child.wait_with_output().ok()?;
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    return None;
                }
            }
        };

        let trimmed_output = output.trim().to_string();
        if trimmed_output.is_empty() {
            None
        } else {
            Some(trimmed_output)
        }
    }

    pub fn clean_menu_output(&self, output: &str, icon_type: &str) -> String {
        let output_trimmed = output.trim();

        if icon_type == "font" {
            output_trimmed
                .chars()
                .skip_while(|c| !c.is_ascii_alphanumeric())
                .collect::<String>()
                .trim()
                .to_string()
        } else if icon_type == "xdg" {
            output_trimmed
                .split('\0')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        } else {
            output_trimmed.to_string()
        }
    }

    pub fn select_network<'a, I>(
        &self,
        mut networks: I,
        output: String,
        icon_type: &str,
        spaces: usize,
    ) -> Option<(Network, i16)>
    where
        I: Iterator<Item = &'a (Network, i16)>,
    {
        let cleaned_output = self.clean_menu_output(&output, icon_type);

        networks
            .find(|(network, signal_strength)| {
                let formatted_network =
                    self.icons
                        .format_network_display(network, *signal_strength, icon_type, spaces);

                let formatted_name = if icon_type == "font" {
                    self.clean_menu_output(&formatted_network, icon_type)
                } else if icon_type == "xdg" {
                    formatted_network
                        .split('\0')
                        .next()
                        .unwrap_or("")
                        .to_string()
                } else {
                    formatted_network
                };

                formatted_name == cleaned_output
            })
            .cloned()
    }

    pub fn prompt_passphrase(
        &self,
        menu_command: &Option<String>,
        ssid: &str,
        icon_type: &str,
    ) -> Option<String> {
        let prompt = format!("Enter passphrase for {}: ", ssid);
        self.run_menu_command(menu_command, None, icon_type)
    }

    pub async fn show_main_menu(
        &self,
        menu_command: &Option<String>,
        station: &mut Station,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<MainMenuOptions>> {
        let options_before_networks = vec![
            ("scan", MainMenuOptions::Scan.to_str()),
            ("known_networks", MainMenuOptions::KnownNetworks.to_str()),
        ];
        let mut input = self
            .icons
            .get_icon_text(options_before_networks, icon_type, spaces);

        for (network, signal_strength) in &station.known_networks {
            let network_info =
                self.icons
                    .format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("\n{}", network_info));
        }

        for (network, signal_strength) in &station.new_networks {
            let network_info =
                self.icons
                    .format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("\n{}", network_info));
        }

        let options_after_networks = vec![("settings", MainMenuOptions::Settings.to_str())];
        let settings_input = self
            .icons
            .get_icon_text(options_after_networks, icon_type, spaces);
        input.push_str(&format!("\n{}", settings_input));

        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type);

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            if let Some(option) = MainMenuOptions::from_str(&cleaned_output) {
                return Ok(Some(option));
            }
        }

        Ok(None)
    }

    pub async fn show_known_networks_menu(
        &self,
        menu_command: &Option<String>,
        station: &mut Station,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<KnownNetwork>> {
        let mut input = String::new();

        for (network, _signal_strength) in &station.known_networks {
            if let Some(ref known_network) = network.known_network {
                let network_info =
                    self.icons
                        .format_known_network_display(known_network, icon_type, spaces);
                input.push_str(&format!("{}\n", network_info));
            }
        }

        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type);

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            let selected_known_network = station
                .known_networks
                .iter()
                .find(|(network, _)| {
                    if let Some(ref known_network) = network.known_network {
                        let formatted_network_name = self.clean_menu_output(
                            &self.icons.format_known_network_display(
                                known_network,
                                icon_type,
                                spaces,
                            ),
                            icon_type,
                        );

                        formatted_network_name == cleaned_output
                    } else {
                        false
                    }
                })
                .and_then(|(network, _)| network.known_network.clone());

            Ok(selected_known_network)
        } else {
            Ok(None)
        }
    }

    pub async fn show_known_network_options(
        &self,
        menu_command: &Option<String>,
        known_network: &KnownNetwork,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<KnownNetworkOptions>> {
        let toggle_autoconnect_option = if known_network.is_autoconnect {
            self.icons.get_icon_text(
                vec![("disable_autoconnect", "Disable Autoconnect")],
                icon_type,
                spaces,
            )
        } else {
            self.icons.get_icon_text(
                vec![("enable_autoconnect", "Enable Autoconnect")],
                icon_type,
                spaces,
            )
        };

        let forget_option = self.icons.get_icon_text(
            vec![("forget_network", "Forget Network")],
            icon_type,
            spaces,
        );

        let input = format!("{}\n{}", toggle_autoconnect_option, forget_option);
        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type);

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if let Some(option) = KnownNetworkOptions::from_str(&cleaned_output) {
                Ok(Some(option))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn show_settings_menu(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<SettingsMenuOptions>> {
        let options = vec![
            (
                SettingsMenuOptions::DisableAdapter.to_id(),
                SettingsMenuOptions::DisableAdapter.to_str(),
            ),
            (
                SettingsMenuOptions::ChangeMode.to_id(),
                SettingsMenuOptions::ChangeMode.to_str(),
            ),
        ];

        let input = self.icons.get_icon_text(options, icon_type, spaces);

        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type);

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if let Some(option) = SettingsMenuOptions::from_str(&cleaned_output) {
                return Ok(Some(option));
            }
        }

        Ok(None)
    }

    pub fn prompt_enable_adapter(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Option<String> {
        let power_on_icon = self.icons.get_icon_text(
            vec![("power_on_device", "Power On Device")],
            icon_type,
            spaces,
        );
        let input = format!("{}\n", power_on_icon);

        self.run_menu_command(menu_command, Some(&input), icon_type)
    }

    pub fn show_change_mode_menu(
        &self,
        menu_command: &Option<String>,
        adapter: &Adapter,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<ChangeModeMenuOptions>> {
        let options = adapter
            .supported_modes
            .iter()
            .filter(|mode| mode == &"station" || mode == &"ap")
            .map(|mode| {
                let (formatted_mode, icon_key) = match mode.as_str() {
                    "station" => ("Station", "station"),
                    "ap" => ("Access Point", "access_point"),
                    _ => (mode.as_str(), ""),
                };
                (icon_key, formatted_mode)
            })
            .collect::<Vec<(&str, &str)>>();

            let input = self.icons.get_icon_text(options, icon_type, spaces);
        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type);

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            let mode_id = match cleaned_output.as_str() {
                "Station" => "station",
                "Access Point" => "ap",
                _ => cleaned_output.as_str(),
            };

            if let Some(option) = ChangeModeMenuOptions::from_id(mode_id) {
                return Ok(Some(option));
            }
        }

        Ok(None)
    }

    pub async fn show_ap_menu(
        &self,
        menu_command: &Option<String>,
        access_point: &AccessPoint,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<ApMenuOptions>> {
        let options = vec![
            if access_point.has_started {
                ("stop_ap", ApMenuOptions::StopAp.to_str())
            } else {
                ("start_ap", ApMenuOptions::StartAp.to_str())
            },
            ("set_ssid", ApMenuOptions::SetSsid.to_str()),
            ("set_password", ApMenuOptions::SetPassword.to_str()),
            ("change_mode", ApMenuOptions::ChangeMode.to_str()),
        ];

        let input = self.icons.get_icon_text(options, icon_type, spaces);
        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type);

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if let Some(option) =
                ApMenuOptions::from_id(&cleaned_output.to_lowercase().replace(" ", "_"))
            {
                return Ok(Some(option));
            }
        }

        Ok(None)
    }

    pub fn prompt_ssid(&self, menu_command: &Option<String>, icon_type: &str) -> Option<String> {
        let prompt_text = "Enter SSID for AP: ";
        self.run_menu_command(menu_command, None, icon_type)
    }

    pub fn prompt_password(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
    ) -> Option<String> {
        let prompt_text = "Enter password for AP: ";
        self.run_menu_command(menu_command, None, icon_type)
    }
}
