use std::sync::{Arc, Mutex};

use anyhow::Result;
use clap::{builder::EnumValueParser, Arg, Command};
use iwmenu::{app::App, menu::Menu, notification::NotificationManager};
use notify_rust::NotificationHandle;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("dmenu")
                .short('d')
                .long("dmenu")
                .takes_value(true)
                .required(true)
                .value_parser(EnumValueParser::<Menu>::new())
                .default_value("dmenu")
                .help("Dmenu backend to use (dmenu, rofi, wofi, fuzzel)"),
        )
        .arg(
            Arg::new("icon")
                .short('i')
                .long("icon")
                .takes_value(true)
                .possible_values(["font", "xdg"])
                .default_value("font")
                .help("Choose the type of icons to use (font or xdg)"),
        )
        .get_matches();

    let menu: Menu = matches.get_one::<Menu>("dmenu").cloned().unwrap();
    let icon_type = matches.get_one::<String>("icon").cloned().unwrap();

    let (log_sender, mut log_receiver) = unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(log) = log_receiver.recv().await {
            println!("LOG: {}", log);
        }
    });

    let (notification_manager, notification_receiver) = NotificationManager::new();
    let notification_manager = Arc::new(notification_manager);
    let notification_handle: Arc<Mutex<Option<NotificationHandle>>> = Arc::new(Mutex::new(None));
    tokio::spawn(NotificationManager::handle_notifications(
        notification_receiver,
        Arc::clone(&notification_handle),
    ));

    let mut app: App = App::new(
        menu.clone(),
        log_sender.clone(),
        Arc::clone(&notification_manager),
    )
    .await?;

    app.run(&menu, &icon_type).await?;

    Ok(())
}
