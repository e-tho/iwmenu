use anyhow::{anyhow, Result};
use notify_rust::{Notification, NotificationHandle, Timeout};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::icons::Icons;

pub struct NotificationManager {
    icons: Arc<Icons>,
    handles: Arc<Mutex<HashMap<u32, NotificationHandle>>>,
}

impl NotificationManager {
    pub fn new(icons: Arc<Icons>) -> Self {
        Self {
            icons,
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_icons_default() -> Self {
        Self::new(Arc::new(Icons::default()))
    }

    pub fn send_notification(
        &self,
        summary: Option<String>,
        body: Option<String>,
        icon: Option<&str>,
        timeout: Option<Timeout>,
    ) -> Result<u32> {
        let icon_name = self.icons.get_xdg_icon(icon.unwrap_or("network_wireless"));

        let mut notification = Notification::new();
        notification
            .summary(summary.as_deref().unwrap_or("iNet Wireless Menu"))
            .body(body.as_deref().unwrap_or(""))
            .icon(&icon_name)
            .timeout(timeout.unwrap_or(Timeout::Milliseconds(3000)));

        let handle = notification.show()?;
        let id = handle.id();

        let mut handles = self
            .handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on notification handles: {}", e))?;
        handles.insert(id, handle);

        Ok(id)
    }

    pub fn close_notification(&self, id: u32) -> Result<()> {
        let mut handles = self
            .handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on notification handles: {}", e))?;

        if let Some(handle) = handles.remove(&id) {
            handle.close();
            Ok(())
        } else {
            Err(anyhow!("Notification ID {} not found", id))
        }
    }
}
