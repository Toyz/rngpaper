use std::time::Duration;
use std::sync::{Mutex, Arc};
use once_cell::sync::OnceCell;
use serde::{Serialize, Deserialize};
use std::env;
use std::fs::OpenOptions;
use fs2::FileExt;
use std::io::Read;
use std::fmt;
use std::fmt::Formatter;

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    // Must start with @ ex (@helba)
    pub collections: Option<Vec<String>>,

    // in minutes
    interval: Option<u64>,

    pub orientation: Option<Orientation>,

    // based on orientation
    pub image_resolution: Option<String>,

    // api key for wallhaven (Required for NSFW)
    pub api_key: Option<String>,

    pub categories: Option<Categories>,

    pub purity: Option<Purity>
}
impl Default for Config {
    fn default() -> Self {
        let orientation = Orientation::Landscape;
        Self {
            collections: Some(vec!["@arkas".to_string()]),
            interval: Some(DEFAULT_INTERVAL),
            orientation: Some(orientation),
            image_resolution: Some(orientation.get_image_resolution().to_string()),
            api_key: None,
            categories: Some(Categories::default()),
            purity: Some(Purity::default())
        }
    }
}
impl Config {
    #[inline]
    pub fn get_interval(&self) -> Duration {
        Duration::from_secs(self.interval.unwrap_or(DEFAULT_INTERVAL) * 60)
    }

    #[inline]
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = Some(interval.as_secs() / 60);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Categories {
    pub general: Option<bool>,
    pub anime: Option<bool>,
    pub people: Option<bool>,
}
impl fmt::Display for Categories {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}",
               self.general.unwrap() as i32,
               self.anime.unwrap() as i32,
               self.people.unwrap() as i32
        )
    }
}
impl Default for Categories {
    fn default() -> Self {
        Self {
            general: Some(false),
            anime: Some(true),
            people: Some(false)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Purity {
    pub sfw: Option<bool>,
    pub sketchy: Option<bool>,
    pub nsfw: Option<bool>,
}
impl fmt::Display for Purity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}",
               self.sfw.unwrap() as i32,
               self.sketchy.unwrap() as i32,
               self.nsfw.unwrap() as i32
        )
    }
}
impl Default for Purity {
    fn default() -> Self {
        Self {
            sfw: Some(true),
            sketchy: Some(false),
            nsfw: Some(false)
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
pub enum Orientation {
    Landscape,
    Portrait,
    Squarish,
}

impl Orientation {
    pub fn get_image_resolution(self) -> &'static str {
        match self {
            Orientation::Landscape => "1920x1080",
            Orientation::Portrait => "1080x1920",
            Orientation::Squarish => "1440x1440"
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Orientation::Landscape => "landscape",
            Orientation::Portrait => "portrait",
            Orientation::Squarish => "squarish"
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

static CONFIG: OnceCell<Arc<Mutex<Config>>> = OnceCell::new();

const DEFAULT_INTERVAL: u64 = 10;

pub fn get_config() -> Arc<Mutex<Config>> {
    CONFIG.get_or_init(|| {
        let config_path = env::current_exe().unwrap().with_file_name("rngpaper.toml");
        let mut config_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(config_path.as_path()).expect("Open config file failed");
        config_file.try_lock_exclusive().expect("Other program running"); // unwrap on purpose
        let mut buf = String::new();
        config_file.read_to_string(&mut buf).expect("Can't read config file content to string");
        let config = toml::from_str::<Config>(&buf).expect("Can't parse config");
        Arc::new(Mutex::new(config))
    }).clone()
}


