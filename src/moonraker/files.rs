

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::MoonrakerError;

/// Custom deserializer that accepts both integer and string values,
/// converting them to a String. Moonraker's file list API returns
/// `modified` as a Unix timestamp integer.
fn modified_deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct ModifiedVisitor;

    impl<'de> de::Visitor<'de> for ModifiedVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer or string timestamp")
        }

        fn visit_i64<E>(self, value: i64) -> Result<String, E> {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<String, E> {
            Ok(value.to_string())
        }

        fn visit_f64<E>(self, value: f64) -> Result<String, E> {
            Ok(value.to_string())
        }

        fn visit_str<E>(self, value: &str) -> Result<String, E> {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(ModifiedVisitor)
}

/// A file entry returned by the Moonraker file list API.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FileEntry {
    /// Filename (e.g. "test.gcode").
    #[serde(rename = "filename")]
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// Last modified timestamp (Unix epoch or ISO string).
    #[serde(deserialize_with = "modified_deserialize")]
    pub modified: String,
}

/// File browser backed by Moonraker's HTTP API.
#[derive(Debug, Clone)]
pub struct FileBrowser {
    http_client: reqwest::Client,
    http_url: String,
    timeout: Duration,
}

impl FileBrowser {
    pub fn new(http_client: reqwest::Client, http_url: String, timeout: Duration) -> Self {
        Self {
            http_client,
            http_url,
            timeout,
        }
    }

    /// List files in the given directory path.
    pub async fn list_files(&self, path: &str) -> Result<Vec<FileEntry>, MoonrakerError> {
        let url = format!("{}/server/files/list", self.http_url);
        let body = serde_json::json!({ "root": "gcodes", "path": path });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        if !status.is_success() {
            return Err(MoonrakerError::HttpError(format!("HTTP {status}: {text}")));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| MoonrakerError::JsonError(e.to_string()))?;

        let files = parsed
            .get("result")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(files)
    }

    /// Delete a file at the given path.
    pub async fn delete_file(&self, path: &str) -> Result<(), MoonrakerError> {
        let url = format!("{}/server/files/delete", self.http_url);
        let body = serde_json::json!({ "root": "gcodes", "path": path });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp
                .text()
                .await
                .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;
            return Err(MoonrakerError::HttpError(format!("HTTP {status}: {text}")));
        }

        Ok(())
    }

    /// Start printing a file.
    pub async fn start_print(&self, path: &str) -> Result<(), MoonrakerError> {
        let url = format!("{}/printer/print/start", self.http_url);
        let body = serde_json::json!({ "filename": path });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp
                .text()
                .await
                .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;
            return Err(MoonrakerError::HttpError(format!("HTTP {status}: {text}")));
        }

        Ok(())
    }
}
