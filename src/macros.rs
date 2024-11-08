#[macro_export]
macro_rules! try_send_notification {
    ($manager:expr, $summary:expr, $body:expr, $icon:expr, $timeout:expr) => {{
        let _ = $manager
            .send_notification($summary, $body, $icon, $timeout)
            .map_err(|e| eprintln!("Notification failed: {}", e));
    }};
}

#[macro_export]
macro_rules! try_send_notification_with_id {
    ($manager:expr, $summary:expr, $body:expr, $icon:expr, $timeout:expr) => {{
        match $manager.send_notification($summary, $body, $icon, $timeout) {
            Ok(id) => Some(id),
            Err(e) => {
                eprintln!("Notification failed: {}", e);
                None
            }
        }
    }};
}

#[macro_export]
macro_rules! try_send_log {
    ($log_sender:expr, $msg:expr) => {{
        $log_sender
            .send($msg)
            .unwrap_or_else(|err| println!("Failed to send log message: {}", err));
    }};
}
