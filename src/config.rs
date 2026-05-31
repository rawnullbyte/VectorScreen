use serde::Deserialize;

use crate::moonraker::multi::PrinterConfig as MultiPrinterConfig;
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct PrinterConfig {
    pub ip: String,
    pub port: u16,
}

impl Default for PrinterConfig {
    fn default() -> Self {
        Self {
            ip: "192.168.1.100".to_string(),
            port: 7125,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level: debug, info, warn, error.
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

/// Application configuration loaded from TOML file.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub printer: PrinterConfig,
    pub logging: LoggingConfig,
    /// Multiple printer configurations for multi-printer support.
    /// Each entry in the TOML uses `[[printers]]` syntax.
    pub printers: Vec<MultiPrinterConfig>,
}

impl Config {
    /// Load configuration from file, with env var overrides.
    ///
    /// Config resolution order:
    /// 1. Defaults
    /// 2. TOML file (path from `VECTOR_SCREEN_CONFIG` env or `/etc/vector-screen.toml`)
    /// 3. Environment variables: `MOONRAKER_HOST`, `MOONRAKER_PORT`, `VECTOR_SCREEN_LOG_LEVEL`
    pub fn load() -> Self {
        let mut config = Self::from_file();

        if let Ok(ip) = std::env::var("MOONRAKER_HOST") {
            config.printer.ip = ip;
        }
        if let Ok(port) = std::env::var("MOONRAKER_PORT") {
            if let Ok(p) = port.parse() {
                config.printer.port = p;
            }
        }
        if let Ok(level) = std::env::var("VECTOR_SCREEN_LOG_LEVEL") {
            config.logging.level = level;
        }

        config
    }

    fn from_file() -> Self {
        let path = std::env::var("VECTOR_SCREEN_CONFIG")
            .unwrap_or_else(|_| "/etc/vector-screen.toml".to_string());

        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                log::warn!("Failed to parse config at {path}: {e}, using defaults");
                Self::default()
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("No config file at {path}, using defaults");
                Self::default()
            }
            Err(e) => {
                log::warn!("Failed to read config at {path}: {e}, using defaults");
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.printer.ip, "192.168.1.100");
        assert_eq!(config.printer.port, 7125);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn parse_minimal_toml() {
        let toml = r#"
[printer]
ip = "10.0.0.5"
port = 8080
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.printer.ip, "10.0.0.5");
        assert_eq!(config.printer.port, 8080);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn parse_full_toml() {
        let toml = r#"
[printer]
ip = "10.0.0.5"
port = 8080

[logging]
level = "debug"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.printer.ip, "10.0.0.5");
        assert_eq!(config.printer.port, 8080);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn parse_empty_toml() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.printer.ip, "192.168.1.100");
        assert_eq!(config.printer.port, 7125);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn log_level_options() {
        for level in &["debug", "info", "warn", "error"] {
            let toml = format!("[logging]\nlevel = \"{level}\"");
            let config: Config = toml::from_str(&toml).unwrap();
            assert_eq!(config.logging.level, *level);
        }
    }

}
