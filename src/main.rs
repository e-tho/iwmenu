use anyhow::{anyhow, Result};
use clap::{builder::EnumValueParser, Arg, Command};
use iwmenu::{
    app::App,
    icons::Icons,
    menu::{Menu, MenuType},
};
use std::{env, sync::Arc};
use sys_locale::get_locale;
use tokio::sync::mpsc::unbounded_channel;

rust_i18n::i18n!("locales");

#[tokio::main]
async fn main() -> Result<()> {
    let locale = get_locale().unwrap_or_else(|| {
        eprintln!("Locale not detected, defaulting to 'en-US'.");
        String::from("en-US")
    });
    rust_i18n::set_locale(&locale);

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
                .value_parser(EnumValueParser::<MenuType>::new())
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

    let menu_type: MenuType = matches.get_one::<MenuType>("menu").cloned().unwrap();
    let menu_command = matches.get_one::<String>("menu_command").cloned();
    let icon_type = matches.get_one::<String>("icon").cloned().unwrap();

    let icons = Arc::new(Icons::new());
    let menu = Menu::new(menu_type, icons.clone());

    let spaces = matches
        .get_one::<String>("spaces")
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| anyhow!("Invalid value for --spaces. Must be a positive integer."))?;

    let (log_sender, mut log_receiver) = unbounded_channel::<String>();

    tokio::spawn(async move {
        while let Some(log) = log_receiver.recv().await {
            println!("LOG: {}", log);
        }
    });

    let mut app = initialize_app(&menu, log_sender.clone(), icons.clone()).await?;

    loop {
        if let Err(err) = app.run(&menu, &menu_command, &icon_type, spaces).await {
            eprintln!("Error during app execution: {:?}", err);
            break;
        }

        if app.reset_mode {
            app = initialize_app(&menu, log_sender.clone(), icons.clone()).await?;
            app.reset_mode = false;
        } else {
            break;
        }
    }

    Ok(())
}

async fn initialize_app(
    menu: &Menu,
    log_sender: tokio::sync::mpsc::UnboundedSender<String>,
    icons: Arc<Icons>,
) -> Result<App> {
    App::new(menu.clone(), log_sender, icons).await
}
