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

    fn get_signal_icon(signal_strength: i16, network_type: &str) -> String {
        let (level1, level2, level3, level4) = match network_type {
            "open" => ('\u{f16cb}', '\u{f16cc}', '\u{f16cd}', '\u{f16ce}'),
            _ => ('\u{f0921}', '\u{f0924}', '\u{f0927}', '\u{f092a}'),
        };

        let icon = match signal_strength {
            -10000..=-7500 => level1,
            -7499..=-5000 => level2,
            -4999..=-2500 => level3,
            _ => level4,
        };

        Self::add_spacing(icon, 10, false)
    }

    fn format_network_display(&self, network: &Network, signal_strength: i16) -> String {
        let signal_icon = Self::get_signal_icon(signal_strength, &network.network_type);

        let connected_icon = if network.is_connected {
            Self::add_spacing('\u{f0133}', 10, true)
        } else {
            String::new()
        };

        format!("{}{}{}", signal_icon, network.name, connected_icon)
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
    ) -> Result<Option<String>> {
        loop {
            let mut input = "Scan\n".to_string();

            for (network, signal_strength) in &station.known_networks {
                let network_info = self.format_network_display(network, *signal_strength);
                input.push_str(&format!("{}\n", network_info));
            }

            for (network, signal_strength) in &station.new_networks {
                let network_info = self.format_network_display(network, *signal_strength);
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
                            self.format_network_display(network, *signal_strength) == output
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
