
use serde::Deserialize;

/// A named printer configuration for multi-printer support.
#[derive(Debug, Clone, Deserialize)]
pub struct PrinterConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl Default for PrinterConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            host: "localhost".to_string(),
            port: 7125,
        }
    }
}

/// Manages multiple printer configurations with one active printer at a time.
pub struct MultiPrinterManager {
    printers: Vec<PrinterConfig>,
    active_index: usize,
}

impl MultiPrinterManager {
    /// Create a new manager with the given list of printers.
    ///
    /// The first printer (index 0) becomes active by default.
    pub fn new(printers: Vec<PrinterConfig>) -> Self {
        Self {
            printers,
            active_index: 0,
        }
    }

    pub fn active(&self) -> Option<&PrinterConfig> {
        self.printers.get(self.active_index)
    }

    pub fn switch_to(&mut self, index: usize) -> Result<(), String> {
        if index >= self.printers.len() {
            return Err(format!(
                "printer index {} out of range (0..{})",
                index,
                self.printers.len()
            ));
        }
        self.active_index = index;
        Ok(())
    }

    pub fn list_printers(&self) -> &[PrinterConfig] {
        &self.printers
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_printers() -> Vec<PrinterConfig> {
        vec![
            PrinterConfig {
                name: "printer-1".to_string(),
                host: "192.168.1.10".to_string(),
                port: 7125,
            },
            PrinterConfig {
                name: "printer-2".to_string(),
                host: "192.168.1.20".to_string(),
                port: 8080,
            },
        ]
    }

    #[test]
    fn multi_create_and_active() {
        let manager = MultiPrinterManager::new(two_printers());
        assert_eq!(manager.list_printers().len(), 2);
        let active = manager.active().expect("should have active printer");
        assert_eq!(active.name, "printer-1");
        assert_eq!(active.host, "192.168.1.10");
        assert_eq!(active.port, 7125);
    }

    #[test]
    fn multi_switch_to_valid_index() {
        let mut manager = MultiPrinterManager::new(two_printers());
        assert!(manager.switch_to(1).is_ok());
        let active = manager.active().expect("should have active printer");
        assert_eq!(active.name, "printer-2");
        assert_eq!(active.host, "192.168.1.20");
        assert_eq!(active.port, 8080);
        assert_eq!(manager.active_index(), 1);
    }

    #[test]
    fn multi_switch_to_invalid_index() {
        let mut manager = MultiPrinterManager::new(two_printers());
        let err = manager.switch_to(5).unwrap_err();
        assert!(err.contains("out of range"));
        // Active printer should remain unchanged.
        assert_eq!(manager.active().unwrap().name, "printer-1");
    }

    #[test]
    fn multi_switch_to_boundary() {
        let mut manager = MultiPrinterManager::new(two_printers());
        assert!(manager.switch_to(1).is_ok());
        assert!(manager.switch_to(2).is_err());
    }

    #[test]
    fn multi_empty_manager() {
        let manager = MultiPrinterManager::new(vec![]);
        assert!(manager.active().is_none());
        assert_eq!(manager.list_printers().len(), 0);
        assert_eq!(manager.active_index(), 0);
    }

    #[test]
    fn multi_empty_manager_switch_fails() {
        let mut manager = MultiPrinterManager::new(vec![]);
        assert!(manager.switch_to(0).is_err());
    }

    #[test]
    fn multi_switch_back_and_forth() {
        let mut manager = MultiPrinterManager::new(two_printers());
        manager.switch_to(1).unwrap();
        assert_eq!(manager.active().unwrap().name, "printer-2");
        manager.switch_to(0).unwrap();
        assert_eq!(manager.active().unwrap().name, "printer-1");
    }

    #[test]
    fn multi_printer_config_default() {
        let p = PrinterConfig::default();
        assert_eq!(p.name, "default");
        assert_eq!(p.host, "localhost");
        assert_eq!(p.port, 7125);
    }

    #[test]
    fn multi_list_printers_returns_all() {
        let printers = two_printers();
        let manager = MultiPrinterManager::new(printers);
        let list = manager.list_printers();
        assert_eq!(list[0].name, "printer-1");
        assert_eq!(list[1].name, "printer-2");
    }
}
