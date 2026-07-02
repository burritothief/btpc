use std::env;
use std::path::PathBuf;

use crate::command::Cli;

pub(super) fn selected_path(cli: &Cli) -> Option<PathBuf> {
    if cli.no_config {
        return None;
    }
    if let Some(path) = &cli.config {
        return Some(path.clone());
    }
    if let Some(path) = env::var_os("BTPC_CONFIG") {
        return Some(PathBuf::from(path));
    }
    default_path(&Environment::current())
}

#[derive(Clone, Debug, Default)]
pub(super) struct Environment {
    pub(super) xdg_config_home: Option<PathBuf>,
    pub(super) home: Option<PathBuf>,
    pub(super) app_data: Option<PathBuf>,
}

impl Environment {
    pub(super) fn current() -> Self {
        Self {
            xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            home: env::var_os("HOME").map(PathBuf::from),
            app_data: env::var_os("APPDATA").map(PathBuf::from),
        }
    }
}

pub(super) fn default_path(environment: &Environment) -> Option<PathBuf> {
    default_path_for(Platform::current(), environment)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(super) enum Platform {
    Linux,
    Macos,
    Windows,
}

impl Platform {
    const fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Self::Windows;
        #[cfg(target_os = "macos")]
        return Self::Macos;
        #[cfg(all(unix, not(target_os = "macos")))]
        return Self::Linux;
    }
}

pub(super) fn default_path_for(platform: Platform, environment: &Environment) -> Option<PathBuf> {
    match platform {
        Platform::Windows => environment
            .app_data
            .as_ref()
            .map(|path| path.join("btpc/config.toml")),
        Platform::Macos => environment
            .xdg_config_home
            .as_ref()
            .map(|path| path.join("btpc/config.toml"))
            .or_else(|| {
                environment
                    .home
                    .as_ref()
                    .map(|path| path.join("Library/Application Support/btpc/config.toml"))
            }),
        Platform::Linux => environment
            .xdg_config_home
            .as_ref()
            .map(|path| path.join("btpc/config.toml"))
            .or_else(|| {
                environment
                    .home
                    .as_ref()
                    .map(|path| path.join(".config/btpc/config.toml"))
            }),
    }
}
