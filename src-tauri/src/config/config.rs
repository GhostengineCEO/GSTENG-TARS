use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use tokio::sync::Mutex;

const ENCRYPTION_KEY: &[u8] = b"gsteng-secret";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    #[serde(default = "AiConfig::default_model")]
    pub preferred_model: String,
    #[serde(default)]
    pub use_cloud: bool,
}

impl AiConfig {
    fn default_model() -> String {
        "local".into()
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self { preferred_model: Self::default_model(), use_cloud: false }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ApiKeys {
    pub openai: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareProfile {
    #[serde(default = "HardwareProfile::default_port")]
    pub port: String,
    #[serde(default = "HardwareProfile::default_baud")]
    pub baud_rate: u32,
}

impl HardwareProfile {
    fn default_port() -> String {
        "/dev/ttyUSB0".into()
    }
    fn default_baud() -> u32 {
        115200
    }
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self { port: Self::default_port(), baud_rate: Self::default_baud() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Personality {
    #[serde(default = "Personality::default_name")]
    pub name: String,
    #[serde(default = "Personality::default_greeting")]
    pub greeting: String,
}

impl Personality {
    fn default_name() -> String {
        "TARS".into()
    }
    fn default_greeting() -> String {
        "Hello".into()
    }
}

impl Default for Personality {
    fn default() -> Self {
        Self { name: Self::default_name(), greeting: Self::default_greeting() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub api_keys: ApiKeys,
    #[serde(default)]
    pub hardware: HardwareProfile,
    #[serde(default)]
    pub personality: Personality,
}

impl Default for Config {
    fn default() -> Self {
        Self { ai: AiConfig::default(), api_keys: ApiKeys::default(), hardware: HardwareProfile::default(), personality: Personality::default() }
    }
}

pub type SharedConfig = Arc<Mutex<Config>>;

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if path.as_ref().exists() {
            let content = fs::read_to_string(&path)?;
            let mut cfg: Config = toml::from_str(&content).map_err(to_io)?;
            cfg.decrypt_keys();
            cfg.validate();
            Ok(cfg)
        } else {
            let cfg = Config::default();
            cfg.save(&path)?;
            Ok(cfg)
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut enc = self.clone();
        enc.encrypt_keys();
        let toml = toml::to_string_pretty(&enc).map_err(to_io)?;
        fs::write(path, toml)
    }

    fn validate(&mut self) {
        if self.ai.preferred_model.is_empty() {
            self.ai.preferred_model = AiConfig::default_model();
        }
        if self.hardware.port.is_empty() {
            self.hardware.port = HardwareProfile::default_port();
        }
        if self.hardware.baud_rate == 0 {
            self.hardware.baud_rate = HardwareProfile::default_baud();
        }
        if self.personality.name.is_empty() {
            self.personality.name = Personality::default_name();
        }
        if self.personality.greeting.is_empty() {
            self.personality.greeting = Personality::default_greeting();
        }
    }

    fn encrypt_keys(&mut self) {
        if let Some(ref mut key) = self.api_keys.openai {
            *key = encrypt(key);
        }
    }

    fn decrypt_keys(&mut self) {
        if let Some(ref mut key) = self.api_keys.openai {
            *key = decrypt(key);
        }
    }
}

fn to_io<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}

fn cipher(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ ENCRYPTION_KEY[i % ENCRYPTION_KEY.len()])
        .collect()
}

fn encrypt(text: &str) -> String {
    base64::encode(cipher(text.as_bytes()))
}

fn decrypt(text: &str) -> String {
    match base64::decode(text) {
        Ok(bytes) => String::from_utf8_lossy(&cipher(&bytes)).into(),
        Err(_) => String::new(),
    }
}

pub fn start_hot_reload(path: PathBuf, cfg: SharedConfig) -> notify::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_)) || matches!(event.kind, EventKind::Create(_)) {
                if let Ok(new_cfg) = Config::load(&path) {
                    let mut lock = cfg.blocking_lock();
                    *lock = new_cfg;
                }
            }
        }
    })?;
    watcher.watch(&path, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
