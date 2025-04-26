#[macro_use]
extern crate rust_i18n;

#[macro_use]
mod macros;

i18n!("locales");

pub mod app;
pub mod icons;
pub mod launcher;
pub mod menu;
pub mod notification;
pub mod iw {
    pub mod access_point;
    pub mod adapter;
    pub mod agent;
    pub mod device;
    pub mod known_network;
    pub mod network;
    pub mod station;
}
