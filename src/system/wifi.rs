
/// Wi-Fi management module for VectorScreen.
///
/// Provides command formatting and output parsing for `nmcli`-based
/// Wi-Fi operations. Does **not** execute any commands — callers are
/// responsible for running the returned strings.
/// Information about a single Wi-Fi network as reported by `nmcli`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInfo {
    pub ssid: String,
    pub signal_strength: i32,
    pub security: String,
    pub connected: bool,
}

/// Stateless helper for building and parsing `nmcli` commands.
///
/// All methods are pure — they return the command string that *would*
/// be executed, plus a `Vec<NetworkInfo>` from scan output.
pub struct WifiManager;

impl WifiManager {
    /// Return the `nmcli` command that lists visible Wi-Fi networks.
    ///
    /// Output format (colon-separated): `SSID,SIGNAL,SECURITY`
    pub fn scan_command() -> String {
        "nmcli -t -f SSID,SIGNAL,SECURITY dev wifi list".to_string()
    }

    /// Return the `nmcli` command that connects to a network.
    pub fn connect_command(ssid: &str, password: &str) -> String {
        format!(
            "nmcli dev wifi connect \"{ssid}\" password \"{password}\""
        )
    }

    /// Return the `nmcli` command that disconnects an interface.
    pub fn disconnect_command(interface: &str) -> String {
        format!("nmcli dev disconnect {interface}")
    }

    /// Parse the colon-separated output of `nmcli -t -f SSID,SIGNAL,SECURITY`.
    ///
    /// Empty lines and entries with an empty SSID (hidden networks) are
    /// silently skipped.
    pub fn parse_scan_output(output: &str) -> Vec<NetworkInfo> {
        output
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }

                let mut parts = line.splitn(3, ':');
                let ssid = parts.next()?;
                if ssid.is_empty() {
                    return None;
                }
                let signal_strength = parts.next()?.parse::<i32>().unwrap_or(0);
                let security = parts.next()?.to_string();

                Some(NetworkInfo {
                    ssid: ssid.to_string(),
                    signal_strength,
                    security,
                    connected: false,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_scan_command() {
        let cmd = WifiManager::scan_command();
        assert_eq!(
            cmd,
            "nmcli -t -f SSID,SIGNAL,SECURITY dev wifi list"
        );
    }

    #[test]
    fn test_connect_command() {
        let cmd = WifiManager::connect_command("MyNetwork", "s3cret");
        assert_eq!(
            cmd,
            "nmcli dev wifi connect \"MyNetwork\" password \"s3cret\""
        );
    }

    #[test]
    fn test_connect_command_special_chars() {
        let cmd = WifiManager::connect_command("Net with spaces!", "p@ss!word#1");
        assert_eq!(
            cmd,
            "nmcli dev wifi connect \"Net with spaces!\" password \"p@ss!word#1\""
        );
    }

    #[test]
    fn test_disconnect_command() {
        let cmd = WifiManager::disconnect_command("wlan0");
        assert_eq!(cmd, "nmcli dev disconnect wlan0");
    }


    #[test]
    fn test_parse_scan_output_multiple() {
        let sample = "\
HomeWiFi:85:WPA2
OfficeNet:72:WPA1 WPA2
Guest:40:
CafeOpen:55:OWE";

        let networks = WifiManager::parse_scan_output(sample);
        assert_eq!(networks.len(), 4);

        assert_eq!(networks[0].ssid, "HomeWiFi");
        assert_eq!(networks[0].signal_strength, 85);
        assert_eq!(networks[0].security, "WPA2");
        assert!(!networks[0].connected);

        assert_eq!(networks[1].ssid, "OfficeNet");
        assert_eq!(networks[1].signal_strength, 72);
        assert_eq!(networks[1].security, "WPA1 WPA2");

        assert_eq!(networks[2].ssid, "Guest");
        assert_eq!(networks[2].signal_strength, 40);
        assert_eq!(networks[2].security, "");

        assert_eq!(networks[3].ssid, "CafeOpen");
        assert_eq!(networks[3].signal_strength, 55);
        assert_eq!(networks[3].security, "OWE");
    }

    #[test]
    fn test_parse_scan_output_single() {
        let sample = "MyNet:99:WPA2";
        let networks = WifiManager::parse_scan_output(sample);
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].ssid, "MyNet");
        assert_eq!(networks[0].signal_strength, 99);
    }

    #[test]
    fn test_parse_scan_output_empty() {
        let networks = WifiManager::parse_scan_output("");
        assert!(networks.is_empty());
    }

    #[test]
    fn test_parse_scan_output_skips_blank_lines() {
        let sample = "Net1:80:WPA2\n\n\nNet2:60:WPA1";
        let networks = WifiManager::parse_scan_output(sample);
        assert_eq!(networks.len(), 2);
        assert_eq!(networks[0].ssid, "Net1");
        assert_eq!(networks[1].ssid, "Net2");
    }

    #[test]
    fn test_parse_scan_output_skips_hidden_networks() {
        let sample = ":0:WPA2\nVisible:70:WPA2";
        let networks = WifiManager::parse_scan_output(sample);
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].ssid, "Visible");
    }

    #[test]
    fn test_parse_scan_output_skips_malformed_lines() {
        let sample = "OnlyTwoFields:50\nGoodNet:80:WPA2";
        let networks = WifiManager::parse_scan_output(sample);
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].ssid, "GoodNet");
    }

    #[test]
    fn test_parse_scan_output_non_numeric_signal() {
        let sample = "BadSignal:abc:WPA2";
        let networks = WifiManager::parse_scan_output(sample);
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].signal_strength, 0);
    }
}
