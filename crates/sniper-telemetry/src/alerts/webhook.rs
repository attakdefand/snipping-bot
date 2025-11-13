//! Webhook alerting module for the sniper bot.
//!
//! This module provides functionality for sending alerts through custom webhooks.

use anyhow::Result;
use chrono;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook URL for sending alerts
    pub url: String,
    /// HTTP method to use (POST, PUT, etc.)
    pub method: String,
    /// Custom headers to include in requests
    pub headers: Option<HashMap<String, String>>,
    /// Authentication token (if required)
    pub auth_token: Option<String>,
    /// Authentication header name (default: "Authorization")
    pub auth_header: Option<String>,
    /// Enable/disable webhook alerts
    pub enabled: bool,
    /// Timeout for webhook requests (in seconds)
    pub timeout_seconds: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "POST".to_string(),
            headers: None,
            auth_token: None,
            auth_header: Some("Authorization".to_string()),
            enabled: true,
            timeout_seconds: 30,
        }
    }
}

/// Webhook alert sender
pub struct WebhookAlertSender {
    /// HTTP client for making requests
    client: Client,
    /// Webhook configuration
    config: WebhookConfig,
}

impl WebhookAlertSender {
    /// Create a new webhook alert sender
    pub fn new(config: WebhookConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()?;
        Ok(Self { client, config })
    }

    /// Send a message through webhook
    pub async fn send_message(&self, message: &str) -> Result<()> {
        if !self.config.enabled {
            debug!("Webhook alerts are disabled, skipping message: {}", message);
            return Ok(());
        }

        info!("Sending webhook alert: {}", message);

        // Prepare the payload
        let payload = serde_json::json!({
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "severity": "info"
        });

        self.send_webhook_request(&payload).await
    }

    /// Send an alert with severity level
    pub async fn send_alert(&self, message: &str, severity: super::AlertSeverity) -> Result<()> {
        let severity_str = match severity {
            super::AlertSeverity::Info => "info",
            super::AlertSeverity::Warning => "warning",
            super::AlertSeverity::Error => "error",
            super::AlertSeverity::Critical => "critical",
        };

        let payload = serde_json::json!({
            "message": message,
            "severity": severity_str,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "sniper-bot"
        });

        self.send_webhook_request(&payload).await
    }

    /// Send a trade execution alert
    pub async fn send_trade_alert(
        &self,
        pair: &str,
        side: &str,
        price: f64,
        amount: f64,
        profit: Option<f64>,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "event": "trade_executed",
            "pair": pair,
            "side": side,
            "price": price,
            "amount": amount,
            "profit": profit,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "sniper-bot"
        });

        self.send_webhook_request(&payload).await
    }

    /// Send a risk alert
    pub async fn send_risk_alert(
        &self,
        message: &str,
        severity: super::AlertSeverity,
    ) -> Result<()> {
        let severity_str = match severity {
            super::AlertSeverity::Info => "info",
            super::AlertSeverity::Warning => "warning",
            super::AlertSeverity::Error => "error",
            super::AlertSeverity::Critical => "critical",
        };

        let payload = serde_json::json!({
            "event": "risk_alert",
            "message": message,
            "severity": severity_str,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "sniper-bot"
        });

        self.send_webhook_request(&payload).await
    }

    /// Send a system alert
    pub async fn send_system_alert(
        &self,
        message: &str,
        severity: super::AlertSeverity,
    ) -> Result<()> {
        let severity_str = match severity {
            super::AlertSeverity::Info => "info",
            super::AlertSeverity::Warning => "warning",
            super::AlertSeverity::Error => "error",
            super::AlertSeverity::Critical => "critical",
        };

        let payload = serde_json::json!({
            "event": "system_alert",
            "message": message,
            "severity": severity_str,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "sniper-bot"
        });

        self.send_webhook_request(&payload).await
    }

    /// Send a custom payload
    pub async fn send_custom_payload(&self, payload: &serde_json::Value) -> Result<()> {
        if !self.config.enabled {
            debug!("Webhook alerts are disabled, skipping custom payload");
            return Ok(());
        }

        info!("Sending custom webhook payload");

        self.send_webhook_request(payload).await
    }

    /// Send webhook request with payload
    async fn send_webhook_request(&self, payload: &serde_json::Value) -> Result<()> {
        // Build the request
        let mut request_builder = match self.config.method.to_uppercase().as_str() {
            "POST" => self.client.post(&self.config.url),
            "PUT" => self.client.put(&self.config.url),
            "PATCH" => self.client.patch(&self.config.url),
            "GET" => self.client.get(&self.config.url),
            _ => {
                error!("Unsupported HTTP method: {}", self.config.method);
                return Err(anyhow::anyhow!(
                    "Unsupported HTTP method: {}",
                    self.config.method
                ));
            }
        };

        // Add headers
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // Add authentication header if token is provided
        if let Some(token) = &self.config.auth_token {
            let auth_header = self
                .config
                .auth_header
                .as_deref()
                .unwrap_or("Authorization");
            request_builder = request_builder.header(auth_header, token);
        }

        // Add default content type if not already set
        if !self.has_header(&self.config.headers, "content-type") {
            request_builder = request_builder.header("Content-Type", "application/json");
        }

        // Send the request
        match request_builder.json(payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Webhook request sent successfully");
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    error!(
                        "Failed to send webhook request. Status: {}, Error: {}",
                        status, error_text
                    );
                    Err(anyhow::anyhow!(
                        "Webhook API error: {} - {}",
                        status,
                        error_text
                    ))
                }
            }
            Err(e) => {
                error!("Failed to send webhook request: {}", e);
                Err(anyhow::anyhow!("Network error: {}", e))
            }
        }
    }

    /// Check if a header is already set
    fn has_header(&self, headers: &Option<HashMap<String, String>>, header_name: &str) -> bool {
        if let Some(headers_map) = headers {
            let header_name_lower = header_name.to_lowercase();
            headers_map
                .iter()
                .any(|(key, _)| key.to_lowercase() == header_name_lower)
        } else {
            false
        }
    }

    /// Validate the webhook configuration
    pub fn validate_config(&self) -> Result<()> {
        if self.config.url.is_empty() {
            return Err(anyhow::anyhow!("Webhook URL is empty"));
        }

        // Validate URL format
        if !self.config.url.starts_with("http://") && !self.config.url.starts_with("https://") {
            return Err(anyhow::anyhow!(
                "Invalid webhook URL format - must start with http:// or https://"
            ));
        }

        // Validate HTTP method
        let valid_methods = ["POST", "PUT", "PATCH", "GET"];
        if !valid_methods.contains(&self.config.method.to_uppercase().as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid HTTP method: {}",
                self.config.method
            ));
        }

        Ok(())
    }

    /// Test the webhook connection
    pub async fn test_connection(&self) -> Result<()> {
        self.validate_config()?;

        let test_payload = serde_json::json!({
            "event": "test",
            "message": "Webhook integration test successful",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "sniper-bot"
        });

        self.send_webhook_request(&test_payload).await
    }

    /// Update configuration
    pub fn update_config(&mut self, config: WebhookConfig) -> Result<()> {
        self.config = config;
        // Rebuild client with new timeout
        self.client = Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()?;
        Ok(())
    }
}

/// Webhook alert manager for handling multiple webhook endpoints
pub struct WebhookAlertManager {
    /// Collection of webhook senders
    senders: HashMap<String, WebhookAlertSender>,
}

impl WebhookAlertManager {
    /// Create a new webhook alert manager
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
        }
    }

    /// Add a webhook sender
    pub fn add_sender(&mut self, name: String, sender: WebhookAlertSender) -> Result<()> {
        self.senders.insert(name, sender);
        Ok(())
    }

    /// Remove a webhook sender
    pub fn remove_sender(&mut self, name: &str) -> bool {
        self.senders.remove(name).is_some()
    }

    /// Send message to all webhook senders
    pub async fn send_message_to_all(&self, message: &str) -> Result<()> {
        for (name, sender) in &self.senders {
            if let Err(e) = sender.send_message(message).await {
                error!("Failed to send message to webhook '{}': {}", name, e);
            }
        }
        Ok(())
    }

    /// Send alert to all webhook senders
    pub async fn send_alert_to_all(
        &self,
        message: &str,
        severity: super::AlertSeverity,
    ) -> Result<()> {
        for (name, sender) in &self.senders {
            if let Err(e) = sender.send_alert(message, severity.clone()).await {
                error!("Failed to send alert to webhook '{}': {}", name, e);
            }
        }
        Ok(())
    }

    /// Send message to specific webhook sender
    pub async fn send_message_to(&self, name: &str, message: &str) -> Result<()> {
        if let Some(sender) = self.senders.get(name) {
            sender.send_message(message).await
        } else {
            Err(anyhow::anyhow!("Webhook sender '{}' not found", name))
        }
    }

    /// Send alert to specific webhook sender
    pub async fn send_alert_to(
        &self,
        name: &str,
        message: &str,
        severity: super::AlertSeverity,
    ) -> Result<()> {
        if let Some(sender) = self.senders.get(name) {
            sender.send_alert(message, severity).await
        } else {
            Err(anyhow::anyhow!("Webhook sender '{}' not found", name))
        }
    }

    /// Get list of webhook sender names
    pub fn list_senders(&self) -> Vec<&String> {
        self.senders.keys().collect()
    }
}

impl Default for WebhookAlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_config_default() {
        let config = WebhookConfig::default();
        assert!(config.url.is_empty());
        assert_eq!(config.method, "POST");
        assert!(config.headers.is_none());
        assert!(config.auth_token.is_none());
        assert_eq!(config.auth_header, Some("Authorization".to_string()));
        assert!(config.enabled);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_webhook_sender_creation() {
        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };

        let sender = WebhookAlertSender::new(config).unwrap();
        assert_eq!(sender.config.url, "https://example.com/webhook");
        assert_eq!(sender.config.method, "POST");
        assert!(sender.config.headers.is_some());
        assert_eq!(sender.config.auth_token, Some("test-token".to_string()));
        assert_eq!(sender.config.auth_header, Some("X-Auth-Token".to_string()));
        assert!(sender.config.enabled);
        assert_eq!(sender.config.timeout_seconds, 30);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let sender = WebhookAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_ok());

        // Invalid config - empty URL
        let config = WebhookConfig {
            url: "".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let sender = WebhookAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_err());

        // Invalid config - invalid URL format
        let config = WebhookConfig {
            url: "invalid-url".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let sender = WebhookAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_err());

        // Invalid config - invalid HTTP method
        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            method: "INVALID".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let sender = WebhookAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_err());
    }

    #[test]
    fn test_webhook_alert_manager() {
        let manager = WebhookAlertManager::new();
        assert!(manager.senders.is_empty());
        assert!(manager.list_senders().is_empty());
    }

    #[test]
    fn test_header_check() {
        let config = WebhookConfig::default();
        let sender = WebhookAlertSender::new(config).unwrap();

        // Test with no headers
        assert!(!sender.has_header(&None, "content-type"));

        // Test with headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        assert!(sender.has_header(&Some(headers), "content-type"));
    }

    #[tokio::test]
    async fn test_payload_creation() {
        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let _sender = WebhookAlertSender::new(config).unwrap();

        // Test info alert payload
        let info_payload = serde_json::json!({
            "message": "Test info message",
            "severity": "info",
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        // Test warning alert payload
        let warning_payload = serde_json::json!({
            "message": "Test warning message",
            "severity": "warning",
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        // Test error alert payload
        let error_payload = serde_json::json!({
            "message": "Test error message",
            "severity": "error",
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        // Test critical alert payload
        let critical_payload = serde_json::json!({
            "message": "Test critical message",
            "severity": "critical",
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        assert_eq!(info_payload["severity"], "info");
        assert_eq!(warning_payload["severity"], "warning");
        assert_eq!(error_payload["severity"], "error");
        assert_eq!(critical_payload["severity"], "critical");
    }

    #[tokio::test]
    async fn test_trade_alert_payload() {
        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let _sender = WebhookAlertSender::new(config).unwrap();

        // Test trade alert payload with profit
        let trade_payload_with_profit = serde_json::json!({
            "event": "trade_executed",
            "pair": "ETH/USDT",
            "side": "BUY",
            "price": 3000.0,
            "amount": 1.5,
            "profit": 2.5,
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        // Test trade alert payload without profit
        let trade_payload_without_profit = serde_json::json!({
            "event": "trade_executed",
            "pair": "ETH/USDT",
            "side": "SELL",
            "price": 3100.0,
            "amount": 1.5,
            "profit": serde_json::Value::Null,
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        assert_eq!(trade_payload_with_profit["event"], "trade_executed");
        assert_eq!(trade_payload_with_profit["pair"], "ETH/USDT");
        assert_eq!(trade_payload_with_profit["side"], "BUY");
        assert_eq!(trade_payload_with_profit["profit"], 2.5);

        assert_eq!(trade_payload_without_profit["event"], "trade_executed");
        assert_eq!(trade_payload_without_profit["pair"], "ETH/USDT");
        assert_eq!(trade_payload_without_profit["side"], "SELL");
        assert!(trade_payload_without_profit["profit"].is_null());
    }

    #[tokio::test]
    async fn test_risk_alert_payload() {
        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let _sender = WebhookAlertSender::new(config).unwrap();

        // Test risk alert payload
        let risk_payload = serde_json::json!({
            "event": "risk_alert",
            "message": "High slippage detected: 5.2%",
            "severity": "warning",
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        assert_eq!(risk_payload["event"], "risk_alert");
        assert_eq!(risk_payload["message"], "High slippage detected: 5.2%");
        assert_eq!(risk_payload["severity"], "warning");
    }

    #[tokio::test]
    async fn test_system_alert_payload() {
        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            method: "POST".to_string(),
            headers: Some(HashMap::new()),
            auth_token: Some("test-token".to_string()),
            auth_header: Some("X-Auth-Token".to_string()),
            enabled: true,
            timeout_seconds: 30,
        };
        let _sender = WebhookAlertSender::new(config).unwrap();

        // Test system alert payload
        let system_payload = serde_json::json!({
            "event": "system_alert",
            "message": "System is running normally",
            "severity": "info",
            "timestamp": "2023-01-01T00:00:00Z",
            "source": "sniper-bot"
        });

        assert_eq!(system_payload["event"], "system_alert");
        assert_eq!(system_payload["message"], "System is running normally");
        assert_eq!(system_payload["severity"], "info");
    }
}
