use anyhow::Result;
use clap::ArgEnum;
use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::iw::{known_network::KnownNetwork, network::Network, station::Station};

#[derive(Debug, Clone, ArgEnum)]
pub enum Menu {
    Fuzzel,
    Wofi,
    Rofi,
    Dmenu,
    Custom,
}

impl Menu {
    fn add_spacing(icon: char, spaces: usize, before: bool) -> String {
        if before {
            format!("{}{}", " ".repeat(spaces), icon)
        } else {
            format!("{}{}", icon, " ".repeat(spaces))
        }
    }

    fn get_signal_icon(
        signal_strength: i16,
        network: &Network,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        if icon_type == "font" {
            let icon_name = match signal_strength {
                -10000..=-7500 => match network.network_type.as_str() {
                    "open" => '\u{f16cb}',
                    _ => '\u{f0921}',
                },
                -7499..=-5000 => match network.network_type.as_str() {
                    "open" => '\u{f16cc}',
                    _ => '\u{f0924}',
                },
                -4999..=-2500 => match network.network_type.as_str() {
                    "open" => '\u{f16cd}',
                    _ => '\u{f0927}',
                },
                _ => match network.network_type.as_str() {
                    "open" => '\u{f16ce}',
                    _ => '\u{f092a}',
                },
            };

            return Self::add_spacing(icon_name, spaces, false);
        }

        let icon_name = match signal_strength {
            -10000..=-7500 => "network-wireless-signal-weak",
            -7499..=-5000 => "network-wireless-signal-ok",
            -4999..=-2500 => "network-wireless-signal-good",
            _ => "network-wireless-signal-excellent",
        };

        let suffix = if network.network_type == "open" {
            "-symbolic"
        } else {
            "-secure-symbolic"
        };

        format!("{}{}", icon_name, suffix)
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
        if icon_type == "font" {
            Self::add_spacing('\u{f16bd}', spaces, false) + &known_network.name
        } else {
            format!(
                "{}\0icon\x1fnetwork-wireless-connected-symbolic",
                known_network.name
            )
        }
    }

    pub fn run_menu_app(
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
        self.run_menu_app(menu_command, &prompt, icon_type)
    }

    pub async fn show_menu(
        &self,
        menu_command: &Option<String>,
        station: &mut Station,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        let scan_icon = match icon_type {
            "font" => format!("{}{}", Self::add_spacing('\u{f46a}', spaces, false), "Scan"),
            "xdg" => "Scan\0icon\x1fview-refresh-symbolic".to_string(),
            _ => "Scan".to_string(),
        };

        let known_networks_icon = match icon_type {
            "font" => format!(
                "{}{}",
                Self::add_spacing('\u{f0134}', spaces, false),
                "Known Networks"
            ),
            "xdg" => "Known Networks\0icon\x1fapp-installed-symbolic".to_string(),
            _ => "Known Networks".to_string(),
        };

        let settings_icon = match icon_type {
            "font" => format!(
                "{}{}",
                Self::add_spacing('\u{f0493}', spaces, false),
                "Settings"
            ),
            "xdg" => "Settings\0icon\x1fpreferences-system-symbolic".to_string(),
            _ => "Settings".to_string(),
        };

        let mut input = format!("{}\n{}\n", scan_icon, known_networks_icon);

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

        input.push_str(&format!("{}\n", settings_icon));

        let menu_output = self.run_menu_app(&menu_command, &input, icon_type);

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

        let menu_output = self.run_menu_app(menu_command, &input, icon_type);

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
        let mut input = String::new();
        let toggle_autoconnect_option = if known_network.is_autoconnect {
            match icon_type {
                "font" => format!(
                    "{}{}",
                    Self::add_spacing('\u{f0547}', spaces, false),
                    "Disable Autoconnect"
                ),
                "xdg" => "Disable Autoconnect\0icon\x1fmedia-playlist-repeat-symbolic".to_string(),
                _ => "Disable Autoconnect".to_string(),
            }
        } else {
            match icon_type {
                "font" => format!(
                    "{}{}",
                    Self::add_spacing('\u{f0547}', spaces, false),
                    "Enable Autoconnect"
                ),
                "xdg" => "Enable Autoconnect\0icon\x1fmedia-playlist-repeat-symbolic".to_string(),
                _ => "Enable Autoconnect".to_string(),
            }
        };

        let forget_option = match icon_type {
            "font" => format!(
                "{}{}",
                Self::add_spacing('\u{f01b4}', spaces, false),
                "Forget Network"
            ),
            "xdg" => "Forget Network\0icon\x1fclose-symbolic".to_string(),
            _ => "Forget Network".to_string(),
        };

        input.push_str(&format!(
            "{}\n{}\n",
            toggle_autoconnect_option, forget_option
        ));

        let menu_output = self.run_menu_app(menu_command, &input, icon_type);

        Ok(menu_output)
    }

    fn get_disable_adapter_icon(icon_type: &str, spaces: usize) -> String {
        match icon_type {
            "font" => format!(
                "{}{}",
                Menu::add_spacing('\u{f092d}', spaces, false),
                "Disable Adapter"
            ),
            "xdg" => {
                "Disable Adapter\0icon\x1fnetwork-wireless-hardware-disabled-symbolic".to_string()
            }
            _ => "Disable Adapter".to_string(),
        }
    }

    pub fn get_settings_icons(&self, icon_type: &str, spaces: usize) -> String {
        let disable_adapter_icon = Self::get_disable_adapter_icon(icon_type, spaces);

        format!("{}", disable_adapter_icon)
    }

    fn get_power_on_device_icon(icon_type: &str, spaces: usize) -> String {
        match icon_type {
            "font" => format!(
                "{}{}",
                Menu::add_spacing('\u{f0425}', spaces, false),
                "Power On Device"
            ),
            "xdg" => "Power On Device\0icon\x1fsystem-shutdown-symbolic".to_string(),
            _ => "Power On Device".to_string(),
        }
    }

    pub fn prompt_enable_adapter(
        &self,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Option<String> {
        let power_on_icon = Self::get_power_on_device_icon(icon_type, spaces);
        let input = format!("{}\n", power_on_icon);

        self.run_menu_app(menu_command, &input, icon_type)
    }
}
