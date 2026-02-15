use crate::icons::Icons;
use crate::iw::{access_point::AccessPoint, network::Network, station::Station};
use crate::launcher::{Launcher, LauncherType};
use anyhow::{anyhow, Result};
use iwdrs::modes::Mode;
use rust_i18n::t;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum MainMenuOptions {
    Scan,
    Settings,
    Network(String),
}

impl MainMenuOptions {
    pub fn from_string(option: &str) -> Option<Self> {
        match option {
            s if s == t!("menus.main.options.scan.name") => Some(MainMenuOptions::Scan),
            s if s == t!("menus.main.options.settings.name") => Some(MainMenuOptions::Settings),
            other => Some(MainMenuOptions::Network(other.to_string())),
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            MainMenuOptions::Scan => t!("menus.main.options.scan.name"),
            MainMenuOptions::Settings => t!("menus.main.options.settings.name"),
            MainMenuOptions::Network(_) => t!("menus.main.options.network.name"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KnownNetworkOptions {
    DisableAutoconnect,
    EnableAutoconnect,
    ForgetNetwork,
    Disconnect,
    Connect,
}

impl KnownNetworkOptions {
    pub fn from_string(option: &str) -> Option<Self> {
        match option {
            s if s == t!("menus.known_network.options.disable_autoconnect.name") => {
                Some(KnownNetworkOptions::DisableAutoconnect)
            }
            s if s == t!("menus.known_network.options.enable_autoconnect.name") => {
                Some(KnownNetworkOptions::EnableAutoconnect)
            }
            s if s == t!("menus.known_network.options.forget_network.name") => {
                Some(KnownNetworkOptions::ForgetNetwork)
            }
            s if s == t!("menus.known_network.options.disconnect.name") => {
                Some(KnownNetworkOptions::Disconnect)
            }
            s if s == t!("menus.known_network.options.connect.name") => {
                Some(KnownNetworkOptions::Connect)
            }
            _ => None,
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            KnownNetworkOptions::DisableAutoconnect => {
                t!("menus.known_network.options.disable_autoconnect.name")
            }
            KnownNetworkOptions::EnableAutoconnect => {
                t!("menus.known_network.options.enable_autoconnect.name")
            }
            KnownNetworkOptions::ForgetNetwork => {
                t!("menus.known_network.options.forget_network.name")
            }
            KnownNetworkOptions::Disconnect => {
                t!("menus.known_network.options.disconnect.name")
            }
            KnownNetworkOptions::Connect => {
                t!("menus.known_network.options.connect.name")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsMenuOptions {
    DisableAdapter,
    SwitchMode,
}

impl SettingsMenuOptions {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "disable_adapter" => Some(SettingsMenuOptions::DisableAdapter),
            "switch_mode" => Some(SettingsMenuOptions::SwitchMode),
            _ => None,
        }
    }

    pub fn to_id(&self) -> &'static str {
        match self {
            SettingsMenuOptions::DisableAdapter => "disable_adapter",
            SettingsMenuOptions::SwitchMode => "switch_mode",
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            SettingsMenuOptions::DisableAdapter => {
                t!("menus.settings.options.disable_adapter.name")
            }
            SettingsMenuOptions::SwitchMode => t!("menus.settings.options.switch_mode.name"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ApMenuOptions {
    StartAp,
    StopAp,
    SetSsid,
    SetPassword,
    Settings,
}

impl ApMenuOptions {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "start_ap" => Some(ApMenuOptions::StartAp),
            "stop_ap" => Some(ApMenuOptions::StopAp),
            "set_ssid" => Some(ApMenuOptions::SetSsid),
            "set_passphrase" => Some(ApMenuOptions::SetPassword),
            "settings" => Some(ApMenuOptions::Settings),
            _ => None,
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        if s == t!("menus.ap.options.start_ap.name") {
            Some(ApMenuOptions::StartAp)
        } else if s == t!("menus.ap.options.stop_ap.name") {
            Some(ApMenuOptions::StopAp)
        } else if s == t!("menus.ap.options.set_ssid.name") {
            Some(ApMenuOptions::SetSsid)
        } else if s == t!("menus.ap.options.set_passphrase.name") {
            Some(ApMenuOptions::SetPassword)
        } else if s == t!("menus.ap.options.settings.name") {
            Some(ApMenuOptions::Settings)
        } else {
            None
        }
    }

    pub fn to_id(&self) -> &'static str {
        match self {
            ApMenuOptions::StartAp => "start_ap",
            ApMenuOptions::StopAp => "stop_ap",
            ApMenuOptions::SetSsid => "set_ssid",
            ApMenuOptions::SetPassword => "set_passphrase",
            ApMenuOptions::Settings => "settings",
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            ApMenuOptions::StartAp => t!("menus.ap.options.start_ap.name"),
            ApMenuOptions::StopAp => t!("menus.ap.options.stop_ap.name"),
            ApMenuOptions::SetSsid => t!("menus.ap.options.set_ssid.name"),
            ApMenuOptions::SetPassword => t!("menus.ap.options.set_passphrase.name"),
            ApMenuOptions::Settings => t!("menus.ap.options.settings.name"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AdapterMenuOptions {
    PowerOnDevice,
}

impl AdapterMenuOptions {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "power_on_device" => Some(AdapterMenuOptions::PowerOnDevice),
            _ => None,
        }
    }

    pub fn to_id(&self) -> &'static str {
        match self {
            AdapterMenuOptions::PowerOnDevice => "power_on_device",
        }
    }

    pub fn from_string(option: &str) -> Option<Self> {
        if option == t!("menus.adapter.options.power_on_device.name") {
            Some(AdapterMenuOptions::PowerOnDevice)
        } else {
            None
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            AdapterMenuOptions::PowerOnDevice => t!("menus.adapter.options.power_on_device.name"),
        }
    }
}

#[derive(Clone)]
pub struct Menu {
    pub menu_type: LauncherType,
    pub icons: Arc<Icons>,
}

impl Menu {
    pub fn new(menu_type: LauncherType, icons: Arc<Icons>) -> Self {
        Self { menu_type, icons }
    }

    pub fn run_launcher(
        &self,
        menu_command: &Option<String>,
        input: Option<&str>,
        icon_type: &str,
        hint: Option<&str>,
        obfuscate: bool,
    ) -> Result<Option<String>> {
        let cmd =
            Launcher::create_command(&self.menu_type, menu_command, icon_type, hint, obfuscate)?;

        Launcher::run(cmd, input)
    }

    pub fn get_signal_icon(
        &self,
        signal_strength: i16,
        is_secure_network: bool,
        icon_type: &str,
    ) -> String {
        let icon_key = match signal_strength {
            -10000..=-7500 => match is_secure_network {
                true => "signal_weak_secure",
                false => "signal_weak_open",
            },
            -7499..=-5000 => match is_secure_network {
                true => "signal_ok_secure",
                false => "signal_ok_open",
            },
            -4999..=-2500 => match is_secure_network {
                true => "signal_good_secure",
                false => "signal_good_open",
            },
            _ => match is_secure_network {
                true => "signal_excellent_secure",
                false => "signal_excellent_open",
            },
        };

        self.icons.get_icon(icon_key, icon_type)
    }

    pub fn format_network_display(
        &self,
        network: &Network,
        signal_strength: i16,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        let signal_icon = self.get_signal_icon(signal_strength, network.is_secure(), icon_type);
        let mut display = network.name.clone();

        if network.is_connected {
            if let Some(connected_icon) = self.icons.get_icon("connected", "generic").chars().next()
            {
                display.push_str(&Icons::format_with_spacing(connected_icon, spaces, true));
            }
        }

        self.icons
            .format_display_with_icon(&display, &signal_icon, icon_type, spaces)
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
                    self.format_network_display(network, *signal_strength, icon_type, spaces);

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

    pub async fn show_main_menu(
        &self,
        menu_command: &Option<String>,
        station: &mut Station,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<MainMenuOptions>> {
        let scan_text = MainMenuOptions::Scan.to_str();
        let options_before_networks = vec![("scan", scan_text.as_ref())];

        let mut input = self
            .icons
            .get_icon_text(options_before_networks, icon_type, spaces);

        for (network, signal_strength) in &station.known_networks {
            let network_info =
                self.format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("\n{network_info}"));
        }

        for (network, signal_strength) in &station.new_networks {
            let network_info =
                self.format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("\n{network_info}"));
        }

        let settings_text = MainMenuOptions::Settings.to_str();
        let options_after_networks = vec![("settings", settings_text.as_ref())];

        let settings_input = self
            .icons
            .get_icon_text(options_after_networks, icon_type, spaces);
        input.push_str(&format!("\n{settings_input}"));

        let menu_output = self.run_launcher(menu_command, Some(&input), icon_type, None, false)?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            if let Some(option) = MainMenuOptions::from_string(&cleaned_output) {
                return Ok(Some(option));
            }
        }

        Ok(None)
    }

    pub async fn show_known_network_options(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
        available_options: Vec<KnownNetworkOptions>,
        network_ssid: &str,
    ) -> Result<Option<KnownNetworkOptions>> {
        let mut input = String::new();

        for option in &available_options {
            let option_text = match option {
                KnownNetworkOptions::Disconnect => self.icons.get_icon_text(
                    vec![(
                        "disconnect",
                        t!("menus.known_network.options.disconnect.name"),
                    )],
                    icon_type,
                    spaces,
                ),
                KnownNetworkOptions::Connect => self.icons.get_icon_text(
                    vec![("connect", t!("menus.known_network.options.connect.name"))],
                    icon_type,
                    spaces,
                ),
                KnownNetworkOptions::DisableAutoconnect => self.icons.get_icon_text(
                    vec![(
                        "disable_autoconnect",
                        t!("menus.known_network.options.disable_autoconnect.name"),
                    )],
                    icon_type,
                    spaces,
                ),
                KnownNetworkOptions::EnableAutoconnect => self.icons.get_icon_text(
                    vec![(
                        "enable_autoconnect",
                        t!("menus.known_network.options.enable_autoconnect.name"),
                    )],
                    icon_type,
                    spaces,
                ),
                KnownNetworkOptions::ForgetNetwork => self.icons.get_icon_text(
                    vec![(
                        "forget_network",
                        t!("menus.known_network.options.forget_network.name"),
                    )],
                    icon_type,
                    spaces,
                ),
            };
            input.push_str(&format!("{option_text}\n"));
        }

        let hint = t!("menus.known_network.hint", ssid = network_ssid);

        let menu_output =
            self.run_launcher(menu_command, Some(&input), icon_type, Some(&hint), false)?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            return Ok(KnownNetworkOptions::from_string(&cleaned_output));
        }

        Ok(None)
    }

    pub async fn show_settings_menu(
        &self,
        menu_command: &Option<String>,
        current_mode: &Mode,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<SettingsMenuOptions>> {
        let target_mode = match current_mode {
            Mode::Station => Mode::Ap,
            Mode::Ap => Mode::Station,
            _ => {
                return Err(anyhow!("Unhandled mode"));
            }
        };

        let target_mode_text = self.get_mode_text(&target_mode);
        let switch_mode_text = t!(
            "menus.settings.options.switch_mode.name",
            mode = target_mode_text
        );

        let switch_mode_icon = match target_mode {
            Mode::Station => "station",
            Mode::Ap => "access_point",
            _ => "switch_mode",
        };

        let options = vec![
            (
                SettingsMenuOptions::DisableAdapter.to_id(),
                self.icons.format_display_with_icon(
                    &SettingsMenuOptions::DisableAdapter.to_str(),
                    &self.icons.get_icon("disable_adapter", icon_type),
                    icon_type,
                    spaces,
                ),
            ),
            (
                SettingsMenuOptions::SwitchMode.to_id(),
                self.icons.format_display_with_icon(
                    &switch_mode_text,
                    &self.icons.get_icon(switch_mode_icon, icon_type),
                    icon_type,
                    spaces,
                ),
            ),
        ];

        let input = options
            .into_iter()
            .map(|(_, formatted_text)| formatted_text)
            .collect::<Vec<String>>()
            .join("\n");

        let menu_output = self.run_launcher(menu_command, Some(&input), icon_type, None, false)?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if cleaned_output == SettingsMenuOptions::DisableAdapter.to_str() {
                return Ok(Some(SettingsMenuOptions::DisableAdapter));
            } else if cleaned_output == switch_mode_text {
                return Ok(Some(SettingsMenuOptions::SwitchMode));
            }
        }

        Ok(None)
    }

    pub fn get_mode_text(&self, mode: &Mode) -> String {
        match mode {
            Mode::Station => t!("modes.station").to_string(),
            Mode::Ap => t!("modes.access_point").to_string(),
            _ => t!("modes.unknown").to_string(),
        }
    }

    pub fn prompt_enable_adapter(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Option<AdapterMenuOptions> {
        let options = vec![(
            AdapterMenuOptions::PowerOnDevice.to_id(),
            AdapterMenuOptions::PowerOnDevice.to_str(),
        )];

        let input = self.icons.get_icon_text(options, icon_type, spaces);

        if let Ok(Some(output)) =
            self.run_launcher(menu_command, Some(&input), icon_type, None, false)
        {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if let Some(option) = AdapterMenuOptions::from_string(&cleaned_output) {
                return Some(option);
            }
        }

        None
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
                ("stop_ap", t!("menus.ap.options.stop_ap.name"))
            } else {
                ("start_ap", t!("menus.ap.options.start_ap.name"))
            },
            ("set_ssid", t!("menus.ap.options.set_ssid.name")),
            ("set_passphrase", t!("menus.ap.options.set_passphrase.name")),
            ("settings", t!("menus.ap.options.settings.name")),
        ];

        let input = self.icons.get_icon_text(options, icon_type, spaces);

        let menu_output = self.run_launcher(menu_command, Some(&input), icon_type, None, false)?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if let Some(option) = ApMenuOptions::from_string(&cleaned_output) {
                return Ok(Some(option));
            }
        }

        Ok(None)
    }

    pub fn prompt_station_passphrase(
        &self,
        menu_command: &Option<String>,
        ssid: &str,
        icon_type: &str,
    ) -> Option<String> {
        let hint_text = t!("menus.main.options.network.hint", ssid = ssid);
        self.run_launcher(menu_command, None, icon_type, Some(&hint_text), true)
            .ok()
            .flatten()
    }

    pub fn prompt_ap_ssid(&self, menu_command: &Option<String>, icon_type: &str) -> Option<String> {
        let hint_text = t!("menus.ap.options.set_ssid.hint");
        self.run_launcher(menu_command, None, icon_type, Some(&hint_text), false)
            .ok()
            .flatten()
    }

    pub fn prompt_ap_passphrase(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
    ) -> Option<String> {
        let hint_text = t!("menus.ap.options.set_passphrase.hint");
        self.run_launcher(menu_command, None, icon_type, Some(&hint_text), true)
            .ok()
            .flatten()
    }
}
