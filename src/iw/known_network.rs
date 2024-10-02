use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use iwdrs::known_netowk::KnownNetwork as IwdKnownNetwork;
use notify_rust::Timeout;
use rust_i18n::t;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::notification::NotificationManager;

#[derive(Debug, Clone)]
pub struct KnownNetwork {
    pub n: IwdKnownNetwork,
    pub name: String,
    pub network_type: String,
    pub is_autoconnect: bool,
    pub is_hidden: bool,
    pub last_connected: Option<DateTime<FixedOffset>>,
}

impl KnownNetwork {
    pub async fn new(n: IwdKnownNetwork) -> Result<Self> {
        let name = n.name().await?;
        let network_type = n.network_type().await?;
        let is_autoconnect = n.get_autoconnect().await?;
        let is_hidden = n.hidden().await?;
        let last_connected = match n.last_connected_time().await {
            Ok(v) => DateTime::parse_from_rfc3339(&v).ok(),
            Err(_) => None,
        };

        Ok(Self {
            n,
            name,
            network_type,
            is_autoconnect,
            is_hidden,
            last_connected,
        })
    }

    pub async fn forget(
        &self,
        sender: UnboundedSender<String>,
        notification_manager: Arc<NotificationManager>,
    ) -> Result<()> {
        match self.n.forget().await {
            Ok(_) => {
                let msg = t!(
                    "notifications.known_networks.forget_network",
                    network_name = self.name
                );
                sender
                    .send(msg.to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                notification_manager.send_notification(
                    None,
                    Some(msg.to_string()),
                    None,
                    Some(Timeout::Milliseconds(3000)),
                );
            }
            Err(e) => {
                let msg = e.to_string();
                sender
                    .send(msg.clone())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                notification_manager.send_notification(
                    None,
                    Some(msg),
                    None,
                    Some(Timeout::Milliseconds(3000)),
                );
            }
        }
        Ok(())
    }

    pub async fn toggle_autoconnect(
        &self,
        sender: UnboundedSender<String>,
        notification_manager: Arc<NotificationManager>,
    ) -> Result<()> {
        if self.is_autoconnect {
            match self.n.set_autoconnect(false).await {
                Ok(_) => {
                    let msg = t!(
                        "notifications.known_networks.disable_autoconnect",
                        network_name = self.name
                    );
                    sender
                        .send(msg.to_string())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    notification_manager.send_notification(
                        None,
                        Some(msg.to_string()),
                        None,
                        Some(Timeout::Milliseconds(3000)),
                    );
                }
                Err(e) => {
                    let msg = e.to_string();
                    sender
                        .send(msg.clone())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    notification_manager.send_notification(
                        None,
                        Some(msg),
                        None,
                        Some(Timeout::Milliseconds(3000)),
                    );
                }
            }
        } else {
            match self.n.set_autoconnect(true).await {
                Ok(_) => {
                    let msg = t!(
                        "notifications.known_networks.enable_autoconnect",
                        network_name = self.name
                    );
                    sender
                        .send(msg.to_string())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    notification_manager.send_notification(
                        None,
                        Some(msg.to_string()),
                        None,
                        Some(Timeout::Milliseconds(3000)),
                    );
                }
                Err(e) => {
                    let msg = e.to_string();
                    sender
                        .send(msg.clone())
                        .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                    notification_manager.send_notification(
                        None,
                        Some(msg),
                        None,
                        Some(Timeout::Milliseconds(3000)),
                    );
                }
            }
        }
        Ok(())
    }
}
