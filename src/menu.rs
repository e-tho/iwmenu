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
pub enum Menu {
    Fuzzel,
    Wofi,
    Rofi,
    Dmenu,
    Custom,
}

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
        font_icons.insert("scan", '\u{f46a}');
        font_icons.insert("known_networks", '\u{f0134}');
        font_icons.insert("settings", '\u{f0493}');
        font_icons.insert("disable_adapter", '\u{f092d}');
        font_icons.insert("power_on_device", '\u{f0425}');
        font_icons.insert("change_mode", '\u{f0fe2}');
        font_icons.insert("start_ap", '\u{f04b8}');
        font_icons.insert("stop_ap", '\u{f028a}');
        font_icons.insert("set_ssid", '\u{f07f8}');
        font_icons.insert("set_password", '\u{f0841}');
        font_icons.insert("enable_autoconnect", '\u{f0547}');
        font_icons.insert("disable_autoconnect", '\u{f0547}');
        font_icons.insert("forget_network", '\u{f01b4}');
        font_icons.insert("connected", '\u{f16bd}');

        xdg_icons.insert("signal_weak", "network-wireless-signal-weak-symbolic");
        xdg_icons.insert("signal_ok", "network-wireless-signal-ok-symbolic");
        xdg_icons.insert("signal_good", "network-wireless-signal-good-symbolic");
        xdg_icons.insert(
            "signal_excellent",
            "network-wireless-signal-excellent-symbolic",
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
        xdg_icons.insert("start_ap", "network-wireless-symbolic");
        xdg_icons.insert("stop_ap", "network-wireless-hardware-disabled-symbolic");
        xdg_icons.insert("set_ssid", "network-wireless-symbolic");
        xdg_icons.insert("set_password", "network-wireless-secure-symbolic");
        xdg_icons.insert("enable_autoconnect", "media-playlist-repeat-symbolic");
        xdg_icons.insert("disable_autoconnect", "media-playlist-repeat-symbolic");
        xdg_icons.insert("forget_network", "close-symbolic");
        xdg_icons.insert("connected", "network-wireless-connected-symbolic");

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
}

impl Menu {
    fn add_spacing(icon: char, spaces: usize, before: bool) -> String {
        if before {
            format!("{}{}", " ".repeat(spaces), icon)
        } else {
            format!("{}{}", icon, " ".repeat(spaces))
        }
    }

    fn format_icon_text(
        &self,
        icon_key: &str,
        text: &str,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        let icons = Icons::new();
        let icon = icons.get_icon(icon_key, icon_type);

        match icon_type {
            "font" => format!("{}{}{}", icon, " ".repeat(spaces), text),
            "xdg" => format!("{}\0icon\x1f{}", text, icon),
            _ => text.to_string(),
        }
    }

    fn get_signal_icon(
        signal_strength: i16,
        network: &Network,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        let icons = Icons::new();
        if icon_type == "font" {
            let icon_key = match signal_strength {
                -10000..=-7500 => match network.network_type.as_str() {
                    "open" => "signal_weak_open",
                    _ => "signal_weak_secure",
                },
                -7499..=-5000 => match network.network_type.as_str() {
                    "open" => "signal_ok_open",
                    _ => "signal_ok_secure",
                },
                -4999..=-2500 => match network.network_type.as_str() {
                    "open" => "signal_good_open",
                    _ => "signal_good_secure",
                },
                _ => match network.network_type.as_str() {
                    "open" => "signal_excellent_open",
                    _ => "signal_excellent_secure",
                },
            };

            if let Some(icon) = icons.get_icon_char(icon_key) {
                return Self::add_spacing(icon, spaces, false);
            } else {
                return String::new();
            }
        }

        let icon_key = match signal_strength {
            -10000..=-7500 => "signal_weak",
            -7499..=-5000 => "signal_ok",
            -4999..=-2500 => "signal_good",
            _ => "signal_excellent",
        };

        icons.get_icon(icon_key, icon_type)
    }

    fn format_network_display(
        network: &Network,
        signal_strength: i16,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        let signal_icon = Self::get_signal_icon(signal_strength, network, icon_type, spaces);
        let connected_icon = if network.is_connected && icon_type == "font" {
            Self::add_spacing('\u{f0133}', spaces, true)
        } else {
            String::new()
        };

        if icon_type == "xdg" {
            format!("{}\0icon\x1f{}", network.name, signal_icon)
        } else {
            format!("{}{}{}", signal_icon, network.name, connected_icon)
        }
    }

    fn format_known_network_display(
        known_network: &KnownNetwork,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        let icons = Icons::new();
        let connected_icon = icons.get_icon("connected", icon_type);

        if icon_type == "font" {
            if let Some(icon) = icons.get_icon_char("connected") {
                format!(
                    "{}{}",
                    Self::add_spacing(icon, spaces, false),
                    known_network.name
                )
            } else {
                known_network.name.clone()
            }
        } else {
            format!("{}\0icon\x1f{}", known_network.name, connected_icon)
        }
    }

    pub fn run_menu_command(
        &self,
        menu_command: &Option<String>,
        input: &str,
        icon_type: &str,
    ) -> Option<String> {
        let output = match self {
            Menu::Fuzzel => {
                let mut command = Command::new("fuzzel");
                command.arg("-d");

                if icon_type == "font" {
                    command.arg("-I");
                }

                command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        child
                            .stdin
                            .as_mut()
                            .unwrap()
                            .write_all(input.as_bytes())
                            .unwrap();
                        let output = child.wait_with_output()?;
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    })
                    .ok()?
            }
            Menu::Wofi => {
                let mut command = Command::new("wofi");
                command.arg("-d").arg("-i");

                if icon_type == "xdg" {
                    command.arg("-I").arg("-m").arg("-q");
                }

                command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        child
                            .stdin
                            .as_mut()
                            .unwrap()
                            .write_all(input.as_bytes())
                            .unwrap();
                        let output = child.wait_with_output()?;
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    })
                    .ok()?
            }
            Menu::Rofi => {
                let mut command = Command::new("rofi");
                command.arg("-m").arg("-1").arg("-dmenu");

                if icon_type == "xdg" {
                    command.arg("-show-icons");
                }

                command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        child
                            .stdin
                            .as_mut()
                            .unwrap()
                            .write_all(input.as_bytes())
                            .unwrap();
                        let output = child.wait_with_output()?;
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    })
                    .ok()?
            }
            Menu::Dmenu => Command::new("dmenu")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    child
                        .stdin
                        .as_mut()
                        .unwrap()
                        .write_all(input.as_bytes())
                        .unwrap();
                    let output = child.wait_with_output()?;
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                })
                .ok()?,
            Menu::Custom => {
                if let Some(cmd) = menu_command {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    let (cmd, args) = parts.split_first().unwrap();
                    let mut command = Command::new(cmd);
                    command.args(args);

                    command
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .and_then(|mut child| {
                            child
                                .stdin
                                .as_mut()
                                .unwrap()
                                .write_all(input.as_bytes())
                                .unwrap();
                            let output = child.wait_with_output()?;
                            Ok(String::from_utf8_lossy(&output.stdout).to_string())
                        })
                        .ok()?
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
        networks
            .find(|(network, signal_strength)| {
                let formatted_network =
                    Self::format_network_display(network, *signal_strength, icon_type, spaces);

                if icon_type == "xdg" {
                    let output_without_icon = output.split('\0').next().unwrap_or("");
                    formatted_network.split('\0').next().unwrap_or("") == output_without_icon
                } else {
                    formatted_network == output
                }
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
        self.run_menu_command(menu_command, &prompt, icon_type)
    }

    pub async fn show_main_menu(
        &self,
        menu_command: &Option<String>,
        station: &mut Station,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        let mut input = format!(
            "{}\n{}\n",
            self.format_icon_text("scan", "Scan", icon_type, spaces),
            self.format_icon_text("known_networks", "Known Networks", icon_type, spaces)
        );

        for (network, signal_strength) in &station.known_networks {
            let network_info =
                Self::format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("{}\n", network_info));
        }

        for (network, signal_strength) in &station.new_networks {
            let network_info =
                Self::format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("{}\n", network_info));
        }

        input.push_str(&self.format_icon_text("settings", "Settings", icon_type, spaces));
        input.push('\n');

        let menu_output = self.run_menu_command(&menu_command, &input, icon_type);
        Ok(menu_output)
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
                    Self::format_known_network_display(known_network, icon_type, spaces);
                input.push_str(&format!("{}\n", network_info));
            }
        }

        let menu_output = self.run_menu_command(menu_command, &input, icon_type);

        if let Some(output) = menu_output {
            let output_without_icon = if icon_type == "xdg" {
                output.split('\0').next().unwrap_or("")
            } else {
                &output
            };

            let selected_known_network = station
                .known_networks
                .iter()
                .find(|(network, _)| {
                    if let Some(ref known_network) = network.known_network {
                        Self::format_known_network_display(known_network, icon_type, spaces)
                            .starts_with(output_without_icon)
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
    ) -> Result<Option<String>> {
        let toggle_autoconnect_option = if known_network.is_autoconnect {
            self.format_icon_text(
                "disable_autoconnect",
                "Disable Autoconnect",
                icon_type,
                spaces,
            )
        } else {
            self.format_icon_text(
                "enable_autoconnect",
                "Enable Autoconnect",
                icon_type,
                spaces,
            )
        };

        let forget_option =
            self.format_icon_text("forget_network", "Forget Network", icon_type, spaces);

        let input = format!("{}\n{}", toggle_autoconnect_option, forget_option);

        let menu_output = self.run_menu_command(menu_command, &input, icon_type);

        Ok(menu_output)
    }

    pub fn get_settings_icons(&self, icon_type: &str, spaces: usize) -> String {
        let disable_adapter_icon =
            self.format_icon_text("disable_adapter", "Disable Adapter", icon_type, spaces);
        let change_mode_icon =
            self.format_icon_text("change_mode", "Change Mode", icon_type, spaces);

        format!("{}\n{}", disable_adapter_icon, change_mode_icon)
    }

    pub fn get_adapter_icons(&self, icon_type: &str, spaces: usize) -> String {
        let disable_adapter_icon =
            self.format_icon_text("disable_adapter", "Disable Adapter", icon_type, spaces);
        let change_mode_icon =
            self.format_icon_text("change_mode", "Change Mode", icon_type, spaces);

        format!("{}\n{}", disable_adapter_icon, change_mode_icon)
    }

    pub fn prompt_enable_adapter(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Option<String> {
        let power_on_icon =
            self.format_icon_text("power_on_device", "Power On Device", icon_type, spaces);
        let input = format!("{}\n", power_on_icon);

        self.run_menu_command(menu_command, &input, icon_type)
    }

    pub fn show_change_mode_menu(
        &self,
        menu_command: &Option<String>,
        adapter: &Adapter,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        let mut input = String::new();
        for mode in &adapter.supported_modes {
            input.push_str(&format!("{}\n", mode));
        }

        let menu_output = self.run_menu_command(menu_command, &input, icon_type);
        Ok(menu_output)
    }

    pub fn get_ap_menu_icons(&self, icon_type: &str, spaces: usize, ap_started: bool) -> String {
        let start_stop_ap_icon = if ap_started {
            self.format_icon_text("stop_ap", "Stop AP", icon_type, spaces)
        } else {
            self.format_icon_text("start_ap", "Start AP", icon_type, spaces)
        };
        let set_ssid_icon = self.format_icon_text("set_ssid", "Set SSID", icon_type, spaces);
        let set_password_icon =
            self.format_icon_text("set_password", "Set Password", icon_type, spaces);

        format!(
            "{}\n{}\n{}",
            start_stop_ap_icon, set_ssid_icon, set_password_icon
        )
    }

    pub fn show_ap_menu(
        &self,
        menu_command: &Option<String>,
        access_point: &AccessPoint,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        let mut input = self.get_ap_menu_icons(icon_type, spaces, access_point.has_started);
        let change_mode_icon =
            self.format_icon_text("change_mode", "Change Mode", icon_type, spaces);
        input.push_str(&format!("\n{}", change_mode_icon));

        let menu_output = self.run_menu_command(menu_command, &input, icon_type);

        Ok(menu_output)
    }

    pub fn prompt_ssid(&self, menu_command: &Option<String>, icon_type: &str) -> Option<String> {
        let prompt = "Enter SSID for AP: ";
        self.run_menu_command(menu_command, prompt, icon_type)
    }

    pub fn prompt_password(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
    ) -> Option<String> {
        let prompt = "Enter password for AP: ";
        self.run_menu_command(menu_command, prompt, icon_type)
    }
}
