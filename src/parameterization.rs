use clap::{Arg, ArgMatches};
use serde::Deserialize;

use std::borrow::Cow;
use std::io::Read;

pub(crate) struct Config {
    pub(crate) xorg: XOrgConfig,
    pub(crate) theme: Theme,
    pub(crate) create_x_server: bool,
    pub(crate) session: Session,
    pub(crate) users: Users,
}
#[derive(Deserialize)]
struct ConfigFile {
    #[serde(default)]
    xorg: XOrgConfig,
    #[serde(default)]
    session: Session,
    #[serde(default)]
    users: Users,
    theme: Theme,
}

fn false_bool() -> bool {
    false
}

fn default_display() -> String {
    ":0".into()
}

fn default_vt() -> String {
    "vt7".into()
}

fn default_home() -> String {
    "/home/".into()
}

fn default_session() -> Option<String> {
    None
}

fn default_sessions_dir() -> String {
    "/usr/share/xsessions".into()
}

#[derive(Deserialize)]
pub(crate) struct Theme {
    pub(crate) path: String,
    #[serde(default = "false_bool")]
    pub(crate) debug: bool,
    #[serde(default = "false_bool")]
    pub(crate) allow_external_resources: bool,
}

#[derive(Deserialize)]
pub(crate) struct Users {
    #[serde(default = "false_bool")]
    pub(crate) hide: bool,
    #[serde(default = "default_home")]
    pub(crate) home_prefix: String,
}

#[derive(Deserialize)]
pub(crate) struct Session {
    #[serde(default = "default_sessions_dir")]
    pub(crate) path: String,
    #[serde(default = "default_session")]
    pub(crate) default: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct XOrgConfig {
    #[serde(default = "default_display")]
    pub(crate) display: String,
    #[serde(default = "default_vt")]
    pub(crate) vt: String,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            path: default_sessions_dir(),
            default: default_session(),
        }
    }
}

impl Default for Users {
    fn default() -> Self {
        Self {
            hide: false,
            home_prefix: default_home(),
        }
    }
}

impl Default for XOrgConfig {
    fn default() -> Self {
        Self {
            display: default_display(),
            vt: default_vt(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ConfigError {
    Io(std::io::Error),
    InvalidConfig(toml::de::Error),
}

impl From<ConfigError> for crate::ProgramError {
    fn from(err: ConfigError) -> crate::ProgramError {
        crate::ProgramError::Config(err)
    }
}

fn get(matches: &ArgMatches, key: &str, or: String) -> String {
    matches
        .value_of(key)
        .map(Cow::Borrowed)
        .unwrap_or(Cow::Owned(or))
        .into_owned()
}

impl Config {
    pub(crate) fn from(app: WebDMApp) -> Result<Self, ConfigError> {
        let matches = app.clap_app.get_matches();

        let path = matches
            .value_of("CONFIG")
            .unwrap_or("/etc/webdm/config.toml");

        let file = std::fs::File::open(path).map_err(ConfigError::Io)?;
        let mut buf_reader = std::io::BufReader::new(file);
        let mut contents = String::new();

        buf_reader
            .read_to_string(&mut contents)
            .map_err(ConfigError::Io)?;

        let config: ConfigFile = toml::from_str(&contents).map_err(ConfigError::InvalidConfig)?;

        Ok(Config {
            create_x_server: !matches.is_present("USE_SERVER"),
            xorg: XOrgConfig {
                display: get(&matches, "DISPLAY", config.xorg.display),
                vt: get(&matches, "VT", config.xorg.vt),
            },
            theme: Theme {
                path: get(&matches, "THEME", config.theme.path),
                debug: matches.is_present("DEBUG") || config.theme.debug,
                allow_external_resources: matches.is_present("INSECURE")
                    || config.theme.allow_external_resources,
            },
            session: Session {
                path: get(&matches, "SESSIONS", config.session.path),
                default: config.session.default,
            },
            users: Users {
                hide: matches.is_present("HIDEUSERS") || config.users.hide,
                home_prefix: get(&matches, "HOME", config.users.home_prefix),
            },
        })
    }
}

pub(crate) struct WebDMApp<'a, 'b> {
    clap_app: clap::App<'a, 'b>,
}

impl<'a, 'b> WebDMApp<'a, 'b> {
    pub(crate) fn from(app: clap::App<'a, 'b>) -> Self {
        WebDMApp {
            clap_app: app
                .arg(
                    Arg::with_name("CONFIG")
                        .short("c")
                        .long("config")
                        .takes_value(true)
                        .help("Sets the configuration file"),
                )
                .arg(
                    Arg::with_name("USE_SERVER")
                        .short("x")
                        .long("existing-x-server")
                        .help("Use an already running x server"),
                )
                .arg(
                    Arg::with_name("DISPLAY")
                        .short("d")
                        .long("display")
                        .takes_value(true)
                        .help("The display to start the x server in"),
                )
                .arg(
                    Arg::with_name("VT")
                        .short("t")
                        .long("virtual-terminal")
                        .takes_value(true)
                        .conflicts_with("USE_SERVER")
                        .help("The virtual console to start the x server in"),
                )
                .arg(
                    Arg::with_name("THEME")
                        .long("theme")
                        .takes_value(true)
                        .help("Path of the directory with the theme website"),
                )
                .arg(
                    Arg::with_name("INSECURE")
                        .long("allow-external-resources")
                        .help("Path of the directory with the theme website"),
                )
                .arg(
                    Arg::with_name("DEBUG")
                        .long("debug")
                        .help("Path of the directory with the theme website"),
                )
                .arg(
                    Arg::with_name("HOME")
                        .takes_value(true)
                        .long("home-prefix")
                        .help("Home prefix for real users"),
                )
                .arg(
                    Arg::with_name("HIDEUSERS")
                        .long("hide-users")
                        .help("Hide users from webkit theme"),
                )
                .arg(
                    Arg::with_name("SESSIONS")
                        .takes_value(true)
                        .short("s")
                        .long("sessions")
                        .help("Directory path where the DesktopEntry xsessions are"),
                ),
        }
    }
}
