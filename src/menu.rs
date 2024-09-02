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
        xdg_icons.insert("start_ap", "media-playback-start-symbolic");
        xdg_icons.insert("stop_ap", "media-playback-stop-symbolic");
        xdg_icons.insert("set_ssid", "edit-paste-symbolic");
        xdg_icons.insert("set_password", "safety-symbolic");
        xdg_icons.insert("enable_autoconnect", "media-playlist-repeat-symbolic");
        xdg_icons.insert("disable_autoconnect", "media-playlist-repeat-symbolic");
        xdg_icons.insert("forget_network", "close-symbolic");
        xdg_icons.insert("connected", "network-wireless-connected-symbolic");
        xdg_icons.insert("known_network", "network-wireless-connected-symbolic");

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
        let connected_icon = if network.is_connected && icon_type == "font" {
            Self::format_with_spacing(
                self.get_connected_icon().unwrap_or_default(),
                spaces,
                true,
            )
        } else {
            String::new()
        };

        self.format_display_with_icon(&network.name, &signal_icon, icon_type, spaces)
            + &connected_icon
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
        input: &str,
        icon_type: &str,
    ) -> Option<String> {
        let output = match self.menu_type {
            MenuType::Fuzzel => {
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
            MenuType::Wofi => {
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
            MenuType::Rofi => {
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
            MenuType::Dmenu => Command::new("dmenu")
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
            MenuType::Custom => {
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
                    self.icons
                        .format_network_display(network, *signal_strength, icon_type, spaces);

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
            "{}\n",
            self.icons.get_icon_text(
                vec![("scan", "Scan"), ("known_networks", "Known Networks")],
                icon_type,
                spaces,
            )
        );

        for (network, signal_strength) in &station.known_networks {
            let network_info =
                self.icons
                    .format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("{}\n", network_info));
        }

        for (network, signal_strength) in &station.new_networks {
            let network_info =
                self.icons
                    .format_network_display(network, *signal_strength, icon_type, spaces);
            input.push_str(&format!("{}\n", network_info));
        }

        input.push_str(&self.icons.get_icon_text(
            vec![("settings", "Settings")],
            icon_type,
            spaces,
        ));
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
                    self.icons
                        .format_known_network_display(known_network, icon_type, spaces);
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
                        self.icons
                            .format_known_network_display(known_network, icon_type, spaces)
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

        let menu_output = self.run_menu_command(menu_command, &input, icon_type);

        Ok(menu_output)
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

        self.run_menu_command(menu_command, &input, icon_type)
    }

    pub fn show_change_mode_menu(
        &self,
        menu_command: &Option<String>,
        adapter: &Adapter,
        icon_type: &str,
    ) -> Result<Option<String>> {
        let mut input = String::new();
        for mode in &adapter.supported_modes {
            input.push_str(&format!("{}\n", mode));
        }

        let menu_output = self.run_menu_command(menu_command, &input, icon_type);
        Ok(menu_output)
    }

    pub fn show_ap_menu(
        &self,
        menu_command: &Option<String>,
        access_point: &AccessPoint,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        let input = self.icons.get_icon_text(
            vec![
                if access_point.has_started {
                    ("stop_ap", "Stop AP")
                } else {
                    ("start_ap", "Start AP")
                },
                ("set_ssid", "Set SSID"),
                ("set_password", "Set Password"),
                ("change_mode", "Change Mode"),
            ],
            icon_type,
            spaces,
        );

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
