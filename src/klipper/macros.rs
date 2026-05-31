
use serde_json::Value;

/// Information about a single Klipper macro.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroInfo {
    pub name: String,
    pub params: Vec<String>,
}

/// A list of macros retrieved from the Moonraker `/printer/gcode/macros` endpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroList {
    macros: Vec<MacroInfo>,
}

impl MacroList {
    /// Parse a Moonraker `/printer/gcode/macros` JSON response into a `MacroList`.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "macros": [
    ///     {"name": "LOAD_FILAMENT", "params": ["SPEED"]},
    ///     {"name": "HOME_ALL", "params": []}
    ///   ]
    /// }
    /// ```
    ///
    /// Returns `None` if the response structure is invalid.
    pub fn parse(response: &Value) -> Option<Self> {
        let macros_array = response.get("macros")?.as_array()?;

        let macros = macros_array
            .iter()
            .filter_map(|entry| {
                let name = entry.get("name")?.as_str()?.to_string();
                let params = entry
                    .get("params")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                Some(MacroInfo { name, params })
            })
            .collect();

        Some(MacroList { macros })
    }

    pub fn get_names(&self) -> Vec<&str> {
        self.macros.iter().map(|m| m.name.as_str()).collect()
    }

    /// Format a command to execute the given macro by name.
    pub fn execute_command(name: &str) -> String {
        format!("RUN_MACRO NAME={name}")
    }

    pub fn is_empty(&self) -> bool {
        self.macros.is_empty()
    }

    pub fn len(&self) -> usize {
        self.macros.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_macro_list() {
        let response = json!({
            "macros": [
                {"name": "LOAD_FILAMENT", "params": ["SPEED"]},
                {"name": "HOME_ALL", "params": []}
            ]
        });

        let list = MacroList::parse(&response).expect("should parse successfully");
        assert_eq!(list.len(), 2);

        let names = list.get_names();
        assert_eq!(names, vec!["LOAD_FILAMENT", "HOME_ALL"]);

        assert_eq!(list.macros[0].params, vec!["SPEED"]);
        assert!(list.macros[1].params.is_empty());
    }

    #[test]
    fn test_get_names() {
        let response = json!({
            "macros": [
                {"name": "LOAD_FILAMENT", "params": ["SPEED"]},
                {"name": "HOME_ALL", "params": []},
                {"name": "DISABLE_STEPPERS", "params": []}
            ]
        });

        let list = MacroList::parse(&response).unwrap();
        let names = list.get_names();
        assert_eq!(names, vec!["LOAD_FILAMENT", "HOME_ALL", "DISABLE_STEPPERS"]);
    }

    #[test]
    fn test_execute_command() {
        assert_eq!(MacroList::execute_command("LOAD_FILAMENT"), "RUN_MACRO NAME=LOAD_FILAMENT");
        assert_eq!(MacroList::execute_command("HOME_ALL"), "RUN_MACRO NAME=HOME_ALL");
        assert_eq!(MacroList::execute_command("CUSTOM_MACRO"), "RUN_MACRO NAME=CUSTOM_MACRO");
    }

    #[test]
    fn test_empty_macro_list() {
        let response = json!({ "macros": [] });

        let list = MacroList::parse(&response).expect("should parse successfully");
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.get_names().is_empty());
    }

    #[test]
    fn test_parse_invalid_response() {
        let response = json!({ "not_macros": [] });
        assert!(MacroList::parse(&response).is_none());

        let response = json!("just a string");
        assert!(MacroList::parse(&response).is_none());
    }
}
