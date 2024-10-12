use notify_rust::{Notification, NotificationHandle, Timeout};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}};

pub struct NotificationManager {
    sender: UnboundedSender<NotificationMessage>,
}

pub struct NotificationMessage {
    summary: Option<String>,
    body: Option<String>,
    icon: Option<String>,
    timeout: Option<Timeout>,
}

impl NotificationManager {
    pub fn new() -> (Self, UnboundedReceiver<NotificationMessage>) {
        let (sender, receiver) = unbounded_channel();
        (Self { sender }, receiver)
    }

    pub fn send_notification(
        &self,
        summary: Option<String>,
        body: Option<String>,
        icon: Option<String>,
        timeout: Option<Timeout>,
    ) -> NotificationHandle {
        let mut binding = Notification::new();
        let notification = binding
            .summary(summary.as_deref().unwrap_or("iNet Wireless"))
            .body(body.as_deref().unwrap_or(""))
            .icon(icon.as_deref().unwrap_or("network-wireless"))
            .timeout(timeout.unwrap_or(Timeout::Milliseconds(3000)));

        notification.show().unwrap()
    }

    pub async fn handle_notifications(
        mut receiver: UnboundedReceiver<NotificationMessage>,
        notification_handle: Arc<Mutex<Option<NotificationHandle>>>,
    ) {
        while let Some(message) = receiver.recv().await {
            let mut notification = Notification::new();

            let summary_str = message.summary.as_deref().unwrap_or("iNet Wireless");
            notification.summary(summary_str);

            if let Some(ref body) = message.body {
                notification.body(body);
            }

            let icon_str = message.icon.as_deref().unwrap_or("network-wireless");
            notification.icon(icon_str);

            notification.timeout(message.timeout.unwrap_or(Timeout::Milliseconds(3000)));

            match notification.show() {
                Ok(handle) => {
                    let mut handle_lock = notification_handle.lock().await;
                    if let Some(existing_handle) = handle_lock.take() {
                        existing_handle.close();
                    }
                    *handle_lock = Some(handle);
                }
                Err(err) => eprintln!("Failed to send notification: {}", err),
            }
        }
    }
}
