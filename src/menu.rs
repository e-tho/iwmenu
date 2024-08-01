use anyhow::Result;
use clap::ArgEnum;
use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::iw::station::Station;

#[derive(Debug, Clone, ArgEnum)]
pub enum Menu {
    Fuzzel,
    Wofi,
    Rofi,
    Dmenu,
}

impl Menu {
    pub async fn select_ssid(&self, station: &Station) -> Result<Option<String>> {
        let mut input = String::new();

        for (network, signal_strength) in &station.new_networks {
            let network_info = format!("{} - {}", network.name, signal_strength);
            input.push_str(&format!("{}\n", network_info));
        }

        for (network, signal_strength) in &station.known_networks {
            let network_info = format!("{} - {}", network.name, signal_strength);
            input.push_str(&format!("{}\n", network_info));
        }

        match self.show_menu(&input) {
            Some(menu_output) => {
                let selected_network = station
                    .new_networks
                    .iter()
                    .chain(station.known_networks.iter())
                    .find(|(network, signal_strength)| {
                        format!("{} - {}", network.name, signal_strength) == menu_output
                    })
                    .map(|(network, _)| network.name.clone());

                Ok(selected_network)
            }
            None => Ok(None),
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
