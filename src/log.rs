use std::sync;

static LEVEL: sync::OnceLock<Level> = sync::OnceLock::new();

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

pub fn level() -> Level {
    *LEVEL.get().unwrap_or(&Level::Debug)
}

pub fn set_level(level: Level) -> anyhow::Result<()> {
    LEVEL
        .set(level)
        .map_err(|_| anyhow::anyhow!("Log level set twice"))
}

#[macro_export]
macro_rules! debug {
    ($($argument:tt)*) => {{
        if $crate::log::level() <= $crate::log::Level::Debug {
            eprintln!($($argument)*);
        }
    }};
}

#[macro_export]
macro_rules! info {
    ($($argument:tt)*) => {{
        if $crate::log::level() <= $crate::log::Level::Info {
            eprintln!($($argument)*);
        }
    }};
}

#[macro_export]
macro_rules! warn {
    ($($argument:tt)*) => {{
        if $crate::log::level() <= $crate::log::Level::Warn {
            eprintln!($($argument)*);
        }
    }};
}

pub use super::warn;
pub use debug;
pub use info;
