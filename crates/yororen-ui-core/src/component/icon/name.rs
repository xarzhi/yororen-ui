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
    Search,

    Arrow(ArrowDirection),
    Check,
    Warning,
    Info,
    Close,
    Maximize(bool),
    Minimize,
    User,
    Pencil,
    Trash,
    File,
    Folder,
}

impl From<IconName> for SharedString {
    fn from(value: IconName) -> SharedString {
        let name: Cow<str> = match value {
            IconName::Search => "search".into(),

            IconName::Arrow(direction) => format!("arrow-{direction}").into(),
            IconName::Check => "check".into(),
            IconName::Warning => "warning".into(),
            IconName::Info => "info".into(),
            IconName::Close => "close".into(),
            IconName::Maximize(i) => format!("maximize-{}", if i { "on" } else { "off" }).into(),
            IconName::Minimize => "minimize".into(),
            IconName::User => "user".into(),
            IconName::Pencil => "pencil".into(),
            IconName::Trash => "trash".into(),
            IconName::File => "file".into(),
            IconName::Folder => "folder".into(),
        };
        format!("icons/{name}.svg").into()
    }
}
