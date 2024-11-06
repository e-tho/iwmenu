use std::collections::HashMap;

#[derive(Clone)]
pub struct Icons {
    generic_icons: HashMap<&'static str, char>,
    font_icons: HashMap<&'static str, char>,
    xdg_icons: HashMap<&'static str, &'static str>,
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
        font_icons.insert("connect", '\u{f16bd}');
        font_icons.insert("disconnect", '\u{f073a}');
        font_icons.insert("scan", '\u{f46a}');
        font_icons.insert("settings", '\u{f0493}');
        font_icons.insert("disable_adapter", '\u{f092d}');
        font_icons.insert("power_on_device", '\u{f0425}');
        font_icons.insert("switch_mode", '\u{f0fe2}');
        font_icons.insert("start_ap", '\u{f040d}');
        font_icons.insert("stop_ap", '\u{f0667}');
        font_icons.insert("set_ssid", '\u{f08d5}');
        font_icons.insert("set_passphrase", '\u{f0bc5}');
        font_icons.insert("enable_autoconnect", '\u{f0547}');
        font_icons.insert("disable_autoconnect", '\u{f0547}');
        font_icons.insert("forget_network", '\u{f01b4}');
        font_icons.insert("station", '\u{f059f}');
        font_icons.insert("access_point", '\u{f0003}');

        font_icons.insert("ok", '\u{f05e1}');
        font_icons.insert("error", '\u{f05d6}');
        font_icons.insert("network_wireless", '\u{f05a9}');

        xdg_icons.insert("signal_weak_open", "network-wireless-signal-weak-symbolic");
        xdg_icons.insert("signal_ok_open", "network-wireless-signal-ok-symbolic");
        xdg_icons.insert("signal_good_open", "network-wireless-signal-good-symbolic");
        xdg_icons.insert(
            "signal_excellent_open",
            "network-wireless-signal-excellent-symbolic",
        );
        xdg_icons.insert(
            "signal_weak_secure",
            "network-wireless-signal-weak-secure-symbolic",
        );
        xdg_icons.insert(
            "signal_ok_secure",
            "network-wireless-signal-ok-secure-symbolic",
        );
        xdg_icons.insert(
            "signal_good_secure",
            "network-wireless-signal-good-secure-symbolic",
        );
        xdg_icons.insert(
            "signal_excellent_secure",
            "network-wireless-signal-excellent-secure-symbolic",
        );
        xdg_icons.insert("scan", "sync-synchronizing-symbolic");
        xdg_icons.insert("settings", "preferences-system-symbolic");
        xdg_icons.insert(
            "disable_adapter",
            "network-wireless-hardware-disabled-symbolic",
        );
        xdg_icons.insert("power_on_device", "system-shutdown-symbolic");
        xdg_icons.insert("switch_mode", "media-playlist-repeat-symbolic");
        xdg_icons.insert("start_ap", "media-playback-start-symbolic");
        xdg_icons.insert("stop_ap", "media-playback-stop-symbolic");
        xdg_icons.insert("set_ssid", "edit-symbolic");
        xdg_icons.insert("set_passphrase", "device-security-symbolic");
        xdg_icons.insert("enable_autoconnect", "on-outline-symbolic");
        xdg_icons.insert("disable_autoconnect", "off-outline-symbolic");
        xdg_icons.insert("forget_network", "minus-symbolic");
        xdg_icons.insert("connected", "network-wireless-connected-symbolic");
        xdg_icons.insert("disconnected", "network-wireless-disconnected-symbolic");
        xdg_icons.insert("connect", "entries-linked-symbolic");
        xdg_icons.insert("disconnect", "entries-unlinked-symbolic");
        xdg_icons.insert("station", "network-workgroup-symbolic");
        xdg_icons.insert("access_point", "network-cellular-symbolic");

        xdg_icons.insert("ok", "emblem-default-symbolic");
        xdg_icons.insert("error", "dialog-error-symbolic");
        xdg_icons.insert("network_wireless", "network-wireless-symbolic");

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
                .map_or(String::new(), |&icon| icon.to_string()),
            "xdg" => self
                .xdg_icons
                .get(key)
                .map_or(String::new(), |&icon| icon.to_string()),
            "generic" => self
                .generic_icons
                .get(key)
                .map_or(String::new(), |&icon| icon.to_string()),
            _ => String::new(),
        }
    }

    pub fn get_xdg_icon(&self, key: &str) -> String {
        self.xdg_icons
            .get(key)
            .map_or(String::new(), |&icon| icon.to_string())
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
        if icon_type == "xdg" {
            format!("{}\0icon\x1f{}", name, icon)
        } else {
            format!("{}{}{}", icon, " ".repeat(spaces), name)
        }
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}
