use std::{borrow::Cow, fmt::Display};

use gpui::SharedString;

pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Display for ArrowDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Up => "up",
                Self::Down => "down",
                Self::Left => "left",
                Self::Right => "right",
            }
        )
    }
}

pub enum IconName {
    Microsoft,
    Minecraft,

    Search,

    Arrow(ArrowDirection),
    Check,
    Warning,
    Close,
    Maximize(bool),
    Minimize,
    Modpack,
    PingIndicator(usize),
    Server,
    User,
}

impl From<IconName> for SharedString {
    fn from(value: IconName) -> SharedString {
        let name: Cow<str> = match value {
            IconName::Microsoft => "microsoft".into(),
            IconName::Minecraft => "minecraft".into(),

            IconName::Search => "search".into(),

            IconName::Arrow(direction) => format!("arrow-{direction}").into(),
            IconName::Check => "check".into(),
            IconName::Warning => "warning".into(),
            IconName::Close => "close".into(),
            IconName::Maximize(i) => format!("maximize-{}", if i { "on" } else { "off" }).into(),
            IconName::Minimize => "minimize".into(),
            IconName::Modpack => "modpack".into(),
            IconName::PingIndicator(i) => format!("ping-indicator-{i}").into(),
            IconName::Server => "server".into(),
            IconName::User => "user".into(),
        };
        format!("icons/{name}.svg").into()
    }
}
