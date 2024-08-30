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
            Arg::new("menu")
                .short('m')
                .long("menu")
                .takes_value(true)
                .required(true)
                .value_parser(EnumValueParser::<Menu>::new())
                .default_value("dmenu")
                .help("Menu application to use (dmenu, rofi, wofi, fuzzel)"),
        )
        .arg(
            Arg::new("menu_command")
                .long("menu-command")
                .takes_value(true)
                .required_if_eq("menu", "custom")
                .help("Menu command to use when --menu is set to custom"),
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
        .arg(
            Arg::new("spaces")
                .short('s')
                .long("spaces")
                .takes_value(true)
                .default_value("1")
                .help("Number of spaces between icon and text when using font icons"),
        )
        .get_matches();

    let menu: Menu = matches.get_one::<Menu>("menu").cloned().unwrap();
    let icon_type = matches.get_one::<String>("icon").cloned().unwrap();
    let spaces = matches
        .get_one::<String>("spaces")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    let menu_command = matches.get_one::<String>("menu_command").cloned();

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

    let mut app = App::new(
        menu.clone(),
        log_sender.clone(),
        Arc::clone(&notification_manager),
    )
    .await?;

    loop {
        app.run(&menu, &menu_command, &icon_type, spaces).await?;

        if app.reset_mode {
            app = App::new(
                menu.clone(),
                log_sender.clone(),
                Arc::clone(&notification_manager),
            )
            .await?;
            app.reset_mode = false;
        } else {
            break;
        }
    }

    Ok(())
}
