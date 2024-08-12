use anyhow::Result;
use clap::ArgEnum;
use notify_rust::Timeout;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::iw::{network::Network, station::Station};

#[derive(Debug, Clone, ArgEnum)]
pub enum Menu {
    Fuzzel,
    Wofi,
    Rofi,
    Dmenu,
}

impl Menu {
    fn add_spacing(icon: char, spaces: usize, before: bool) -> String {
        if before {
            format!("{}{}", " ".repeat(spaces), icon)
        } else {
            format!("{}{}", icon, " ".repeat(spaces))
        }
    }

    fn get_signal_icon(signal_strength: i16, network: &Network, icon_type: &str) -> String {
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

            return Self::add_spacing(icon_name, 10, false);
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

    fn format_network_display(network: &Network, signal_strength: i16, icon_type: &str) -> String {
        let signal_icon = Self::get_signal_icon(signal_strength, network, icon_type);

        let connected_icon = if network.is_connected && icon_type == "font" {
            Self::add_spacing('\u{f0133}', 10, true)
        } else {
            String::new()
        };

        if icon_type == "xdg" {
            format!("{}\0icon\x1f{}", network.name, signal_icon)
        } else {
            format!("{}{}{}", signal_icon, network.name, connected_icon)
        }
    }

    pub async fn select_ssid(
        &self,
        station: &mut Station,
        log_sender: UnboundedSender<String>,
        notification_sender: UnboundedSender<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<Timeout>,
        )>,
        icon_type: &str,
    ) -> Result<Option<String>> {
        loop {
            let mut input = "Scan\n".to_string();

            for (network, signal_strength) in &station.known_networks {
                let network_info =
                    Self::format_network_display(network, *signal_strength, icon_type);
                input.push_str(&format!("{}\n", network_info));
            }

            for (network, signal_strength) in &station.new_networks {
                let network_info =
                    Self::format_network_display(network, *signal_strength, icon_type);
                input.push_str(&format!("{}\n", network_info));
            }

            let menu_output = self.show_menu(&input);

            match menu_output {
                Some(output) if output == "Scan" => {
                    station
                        .scan(log_sender.clone(), notification_sender.clone())
                        .await?;
                    station.refresh().await?;
                    continue;
                }
                Some(output) => {
                    let selected_network = station
                        .new_networks
                        .iter()
                        .chain(station.known_networks.iter())
                        .find(|(network, signal_strength)| {
                            let formatted_network =
                                Self::format_network_display(network, *signal_strength, icon_type);

                            if icon_type == "xdg" {
                                let output_without_icon = output.split('\0').next().unwrap_or("");
                                formatted_network.split('\0').next().unwrap_or("")
                                    == output_without_icon
                            } else {
                                formatted_network == output
                            }
                        })
                        .map(|(network, _)| network.name.clone());

                    return Ok(selected_network);
                }
                None => return Ok(None),
            }
        }
    }

    fn show_menu(&self, input: &str) -> Option<String> {
        let output = match self {
            Menu::Fuzzel => Command::new("fuzzel")
                .arg("-d")
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
            Menu::Wofi => Command::new("wofi")
                .arg("-d")
                .arg("-I")
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
            Menu::Rofi => Command::new("rofi")
                .arg("-m")
                .arg("-1")
                .arg("-dmenu")
                .arg("-i")
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
        };

        let trimmed_output = output.trim().to_string();
        if trimmed_output.is_empty() {
            None
        } else {
            Some(trimmed_output)
        }
    }

    pub fn prompt_passphrase(&self, ssid: &str) -> Option<String> {
        let prompt = format!("Enter passphrase for {}: ", ssid);
        self.show_menu(&prompt)
    }
}
