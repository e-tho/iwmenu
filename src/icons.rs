use std::collections::HashMap;

#[derive(Clone)]
pub struct IconDefinition {
    single: String,
    list: String,
}

impl IconDefinition {
    pub fn simple(icon: &str) -> Self {
        Self {
            single: icon.to_string(),
            list: icon.to_string(),
        }
    }

    pub fn with_fallbacks(single: Option<&str>, list: &str) -> Self {
        let single_icon = match single {
            Some(icon) => icon.to_string(),
            None => list.split(',').next().unwrap_or("").trim().to_string(),
        };

        Self {
            single: single_icon,
            list: list.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Icons {
    generic_icons: HashMap<&'static str, char>,
    font_icons: HashMap<&'static str, char>,
    xdg_icons: HashMap<&'static str, IconDefinition>,
}

impl Icons {
    pub fn new() -> Self {
        let mut generic_icons = HashMap::new();
        let mut font_icons = HashMap::new();
        let mut xdg_icons = HashMap::new();

        generic_icons.insert("connected", '\u{23FA}');

        font_icons.insert("signal_weak_open", '\u{f16cb}');
        font_icons.insert("signal_weak_secure", '\u{f0921}');
        font_icons.insert("signal_ok_open", '\u{f16cc}');
        font_icons.insert("signal_ok_secure", '\u{f0924}');
        font_icons.insert("signal_good_open", '\u{f16cd}');
        font_icons.insert("signal_good_secure", '\u{f0927}');
        font_icons.insert("signal_excellent_open", '\u{f16ce}');
        font_icons.insert("signal_excellent_secure", '\u{f092a}');
        font_icons.insert("connected", '\u{f05a9}');
        font_icons.insert("disconnected", '\u{f16bc}');
        font_icons.insert("connect", '\u{f0337}');
        font_icons.insert("disconnect", '\u{f0338}');
        font_icons.insert("scan", '\u{f46a}');
        font_icons.insert("settings", '\u{f0493}');
        font_icons.insert("disable_adapter", '\u{f092d}');
        font_icons.insert("power_on_device", '\u{f0425}');
        font_icons.insert("switch_mode", '\u{f0fe2}');
        font_icons.insert("start_ap", '\u{f040d}');
        font_icons.insert("stop_ap", '\u{f0667}');
        font_icons.insert("set_ssid", '\u{f08d5}');
        font_icons.insert("set_passphrase", '\u{f0bc5}');
        font_icons.insert("enable_autoconnect", '\u{f006a}');
        font_icons.insert("disable_autoconnect", '\u{f19e7}');
        font_icons.insert("forget_network", '\u{f0377}');
        font_icons.insert("station", '\u{f059f}');
        font_icons.insert("access_point", '\u{f0003}');

        font_icons.insert("ok", '\u{f05e1}');
        font_icons.insert("error", '\u{f05d6}');
        font_icons.insert("network_wireless", '\u{f05a9}');

        xdg_icons.insert(
            "signal_weak_open",
            IconDefinition::with_fallbacks(
                None,
                "network-wireless-signal-weak-symbolic,network-wireless-symbolic",
            ),
        );
        xdg_icons.insert(
            "signal_ok_open",
            IconDefinition::with_fallbacks(
                None,
                "network-wireless-signal-ok-symbolic,network-wireless-symbolic",
            ),
        );
        xdg_icons.insert(
            "signal_good_open",
            IconDefinition::with_fallbacks(
                None,
                "network-wireless-signal-good-symbolic,network-wireless-symbolic",
            ),
        );
        xdg_icons.insert(
            "signal_excellent_open",
            IconDefinition::with_fallbacks(
                None,
                "network-wireless-signal-excellent-symbolic,network-wireless-symbolic",
            ),
        );

        xdg_icons.insert("signal_weak_secure", 
        IconDefinition::with_fallbacks(
                Some("network-wireless-signal-weak-symbolic"),
                "network-wireless-signal-weak-secure-symbolic,network-wireless-signal-weak-symbolic,network-wireless-symbolic"
            )
        );
        xdg_icons.insert("signal_ok_secure", 
            IconDefinition::with_fallbacks(
                Some("network-wireless-signal-ok-symbolic"),
                "network-wireless-signal-ok-secure-symbolic,network-wireless-signal-ok-symbolic,network-wireless-symbolic"
            )
        );
        xdg_icons.insert("signal_good_secure", 
            IconDefinition::with_fallbacks(
                Some("network-wireless-signal-good-symbolic"),
                "network-wireless-signal-good-secure-symbolic,network-wireless-signal-good-symbolic,network-wireless-symbolic"
            )
        );
        xdg_icons.insert("signal_excellent_secure", 
            IconDefinition::with_fallbacks(
                Some("network-wireless-signal-excellent-symbolic"),
                "network-wireless-signal-excellent-secure-symbolic,network-wireless-signal-excellent-symbolic,network-wireless-symbolic"
            )
        );

        xdg_icons.insert(
            "scan",
            IconDefinition::with_fallbacks(
                Some("view-refresh-symbolic"),
                "sync-synchronizing-symbolic,emblem-synchronizing,view-refresh-symbolic",
            ),
        );
        xdg_icons.insert("disable_adapter", 
            IconDefinition::with_fallbacks(
                Some("network-wireless-disabled-symbolic"),
                "network-wireless-hardware-disabled-symbolic,network-wireless-disabled-symbolic,network-wireless-off"
            )
        );
        xdg_icons.insert(
            "set_passphrase",
            IconDefinition::with_fallbacks(
                Some("dialog-password-symbolic"),
                "device-security-symbolic,dialog-password-symbolic,changes-prevent",
            ),
        );
        xdg_icons.insert(
            "enable_autoconnect",
            IconDefinition::with_fallbacks(
                None,
                "media-playlist-repeat-symbolic,media-repeat-symbolic",
            ),
        );
        xdg_icons.insert("disable_autoconnect", 
            IconDefinition::with_fallbacks(
                Some("media-playlist-repeat-song-symbolic"),
                "media-playlist-no-repeat-symbolic,media-repeat-none-symbolic,media-playlist-repeat-song-symbolic"
            )
        );
        xdg_icons.insert(
            "connected",
            IconDefinition::with_fallbacks(
                Some("network-wireless-symbolic"),
                "network-wireless-connected-symbolic,network-wireless-symbolic",
            ),
        );
        xdg_icons.insert(
            "disconnected",
            IconDefinition::with_fallbacks(
                Some("network-wireless-offline-symbolic"),
                "network-wireless-disconnected-symbolic,network-wireless-offline-symbolic",
            ),
        );
        xdg_icons.insert(
            "connect",
            IconDefinition::with_fallbacks(
                None,
                "network-connect-symbolic,entries-linked-symbolic,link-symbolic",
            ),
        );
        xdg_icons.insert(
            "disconnect",
            IconDefinition::with_fallbacks(
                None,
                "network-disconnect-symbolic,entries-unlinked-symbolic,media-eject-symbolic",
            ),
        );

        xdg_icons.insert(
            "settings",
            IconDefinition::simple("preferences-system-symbolic"),
        );
        xdg_icons.insert(
            "power_on_device",
            IconDefinition::simple("system-shutdown-symbolic"),
        );
        xdg_icons.insert(
            "switch_mode",
            IconDefinition::simple("media-playlist-repeat-symbolic"),
        );
        xdg_icons.insert(
            "start_ap",
            IconDefinition::simple("media-playback-start-symbolic"),
        );
        xdg_icons.insert(
            "stop_ap",
            IconDefinition::simple("media-playback-stop-symbolic"),
        );
        xdg_icons.insert("set_ssid", IconDefinition::simple("edit-symbolic"));
        xdg_icons.insert(
            "forget_network",
            IconDefinition::simple("list-remove-symbolic"),
        );
        xdg_icons.insert(
            "station",
            IconDefinition::simple("network-wireless-symbolic"),
        );
        xdg_icons.insert(
            "access_point",
            IconDefinition::simple("network-wireless-hotspot-symbolic"),
        );
        xdg_icons.insert("ok", IconDefinition::simple("emblem-default-symbolic"));
        xdg_icons.insert("error", IconDefinition::simple("dialog-error-symbolic"));
        xdg_icons.insert(
            "network_wireless",
            IconDefinition::simple("network-wireless-symbolic"),
        );

        Icons {
            font_icons,
            xdg_icons,
            generic_icons,
        }
    }

    pub fn get_icon(&self, key: &str, icon_type: &str) -> String {
        match icon_type {
            "font" => self
                .font_icons
                .get(key)
                .map(|&icon| icon.to_string())
                .unwrap_or_default(),
            "xdg" => self
                .xdg_icons
                .get(key)
                .map(|icon_definition| icon_definition.list.clone())
                .unwrap_or_default(),
            "generic" => self
                .generic_icons
                .get(key)
                .map(|&icon| icon.to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    pub fn get_xdg_icon(&self, key: &str) -> String {
        self.xdg_icons
            .get(key)
            .map(|icon_def| icon_def.single.clone())
            .unwrap_or_default()
    }

    pub fn get_xdg_icon_list(&self, key: &str) -> String {
        self.xdg_icons
            .get(key)
            .map(|icon_def| icon_def.list.clone())
            .unwrap_or_default()
    }

    pub fn get_icon_text<T>(&self, items: Vec<(&str, T)>, icon_type: &str, spaces: usize) -> String
    where
        T: AsRef<str>,
    {
        items
            .into_iter()
            .map(|(icon_key, text)| {
                let icon = self.get_icon(icon_key, icon_type);
                let text = text.as_ref();
                match icon_type {
                    "font" => format!("{}{}{}", icon, " ".repeat(spaces), text),
                    "xdg" => format!("{}\0icon\x1f{}", text, icon),
                    _ => text.to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn format_with_spacing(icon: char, spaces: usize, before: bool) -> String {
        if before {
            format!("{}{}", " ".repeat(spaces), icon)
        } else {
            format!("{}{}", icon, " ".repeat(spaces))
        }
    }

    pub fn format_display_with_icon(
        &self,
        name: &str,
        icon: &str,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        match icon_type {
            "xdg" => format!("{}\0icon\x1f{}", name, icon),
            "font" | "generic" => format!("{}{}{}", icon, " ".repeat(spaces), name),
            _ => name.to_string(),
        }
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}
