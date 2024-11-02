use anyhow::{anyhow, Result};
use notify_rust::{error::Error as NotifyError, Notification, NotificationHandle, Timeout};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct NotificationManager {
    handles: Arc<Mutex<HashMap<u32, NotificationHandle>>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn send_notification(
        &self,
        summary: Option<String>,
        body: Option<String>,
        icon: Option<String>,
        timeout: Option<Timeout>,
    ) -> Result<u32, NotifyError> {
        let mut notification = Notification::new();

        notification
            .summary(summary.as_deref().unwrap_or("iNet Wireless Menu"))
            .body(body.as_deref().unwrap_or(""))
            .icon(icon.as_deref().unwrap_or("network-wireless-symbolic"))
            .timeout(timeout.unwrap_or(Timeout::Milliseconds(3000)));

        let handle = notification.show()?;

        let id = handle.id();
        self.handles.lock().unwrap().insert(id, handle);

        Ok(id)
    }

    pub fn close_notification(&self, id: u32) -> Result<()> {
        let mut handles = self.handles.lock().unwrap();

        if let Some(handle) = handles.remove(&id) {
            handle.close();
            Ok(())
        } else {
            Err(anyhow!("Notification ID {} not found", id))
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
