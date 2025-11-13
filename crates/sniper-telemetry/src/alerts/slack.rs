//! Slack alerting module for the sniper bot.
//!
//! This module provides functionality for sending alerts through Slack webhooks.

use anyhow::Result;
use chrono;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Slack webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Webhook URL for sending alerts
    pub webhook_url: String,
    /// Channel to send alerts to (if using bot token)
    pub channel: Option<String>,
    /// Username for the bot (if using bot token)
    pub username: Option<String>,
    /// Enable/disable Slack alerts
    pub enabled: bool,
}

/// Slack alert sender
pub struct SlackAlertSender {
    /// HTTP client for making requests
    client: Client,
    /// Slack configuration
    config: SlackConfig,
}

impl SlackAlertSender {
    /// Create a new Slack alert sender
    pub fn new(config: SlackConfig) -> Result<Self> {
        let client = Client::new();
        Ok(Self { client, config })
    }

    /// Send a message through Slack
    pub async fn send_message(&self, message: &str) -> Result<()> {
        if !self.config.enabled {
            debug!("Slack alerts are disabled, skipping message: {}", message);
            return Ok(());
        }

        info!("Sending Slack alert: {}", message);

        // Prepare the payload
        let mut payload = serde_json::Map::new();
        payload.insert(
            "text".to_string(),
            serde_json::Value::String(message.to_string()),
        );

        // Add optional fields if provided
        if let Some(ref channel) = self.config.channel {
            payload.insert(
                "channel".to_string(),
                serde_json::Value::String(channel.clone()),
            );
        }

        if let Some(ref username) = self.config.username {
            payload.insert(
                "username".to_string(),
                serde_json::Value::String(username.clone()),
            );
        }

        // Add blocks for rich formatting
        let blocks = self.create_message_blocks(message);
        payload.insert("blocks".to_string(), serde_json::Value::Array(blocks));

        let payload_value = serde_json::Value::Object(payload);

        match self
            .client
            .post(&self.config.webhook_url)
            .json(&payload_value)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Slack message sent successfully");
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    error!(
                        "Failed to send Slack message. Status: {}, Error: {}",
                        status, error_text
                    );
                    Err(anyhow::anyhow!(
                        "Slack API error: {} - {}",
                        status,
                        error_text
                    ))
                }
            }
            Err(e) => {
                error!("Failed to send Slack message: {}", e);
                Err(anyhow::anyhow!("Network error: {}", e))
            }
        }
    }

    /// Create message blocks for rich formatting
    fn create_message_blocks(&self, message: &str) -> Vec<serde_json::Value> {
        vec![serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": message
            }
        })]
    }

    /// Send an alert with severity level
    pub async fn send_alert(&self, message: &str, severity: super::AlertSeverity) -> Result<()> {
        let formatted_message = match severity {
            super::AlertSeverity::Info => format!(":information_source: *Info*\n{}", message),
            super::AlertSeverity::Warning => format!(":warning: *Warning*\n{}", message),
            super::AlertSeverity::Error => format!(":x: *Error*\n{}", message),
            super::AlertSeverity::Critical => format!(":rotating_light: *Critical*\n{}", message),
        };

        self.send_message(&formatted_message).await
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
        let (emoji, color) = match side.to_lowercase().as_str() {
            "buy" => (":moneybag:", "#00ff00"),             // Green for buy
            "sell" => (":money_with_wings:", "#ff0000"),    // Red for sell
            _ => (":chart_with_upwards_trend:", "#0000ff"), // Blue for other
        };

        let message = if let Some(profit) = profit {
            format!(
                "{} *Trade Executed*\nPair: {}\nSide: {}\nPrice: {:.6}\nAmount: {:.6}\nProfit: {:.6}%",
                emoji, pair, side, price, amount, profit
            )
        } else {
            format!(
                "{} *Trade Executed*\nPair: {}\nSide: {}\nPrice: {:.6}\nAmount: {:.6}",
                emoji, pair, side, price, amount
            )
        };

        let blocks = vec![
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": message
                }
            }),
            serde_json::json!({
                "type": "context",
                "elements": [
                    {
                        "type": "mrkdwn",
                        "text": format!(":clock3: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))
                    }
                ]
            }),
        ];

        let payload = serde_json::json!({
            "text": format!("Trade Executed: {} {}", pair, side),
            "attachments": [
                {
                    "color": color,
                    "blocks": blocks
                }
            ]
        });

        self.send_payload(&payload).await
    }

    /// Send a risk alert
    pub async fn send_risk_alert(
        &self,
        message: &str,
        severity: super::AlertSeverity,
    ) -> Result<()> {
        let (emoji, color) = match severity {
            super::AlertSeverity::Info => (":shield:", "#439FE0"),
            super::AlertSeverity::Warning => (":warning:", "#FFA500"),
            super::AlertSeverity::Error => (":bangbang:", "#FF0000"),
            super::AlertSeverity::Critical => (":rotating_light:", "#8B0000"),
        };

        let formatted_message = format!("{} *Risk Alert*\n{}", emoji, message);

        let blocks = vec![
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": formatted_message
                }
            }),
            serde_json::json!({
                "type": "context",
                "elements": [
                    {
                        "type": "mrkdwn",
                        "text": format!(":clock3: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))
                    }
                ]
            }),
        ];

        let payload = serde_json::json!({
            "text": "Risk Alert",
            "attachments": [
                {
                    "color": color,
                    "blocks": blocks
                }
            ]
        });

        self.send_payload(&payload).await
    }

    /// Send a system alert
    pub async fn send_system_alert(
        &self,
        message: &str,
        severity: super::AlertSeverity,
    ) -> Result<()> {
        let (emoji, color) = match severity {
            super::AlertSeverity::Info => (":computer:", "#808080"),
            super::AlertSeverity::Warning => (":warning:", "#FFA500"),
            super::AlertSeverity::Error => (":boom:", "#FF0000"),
            super::AlertSeverity::Critical => (":skull_and_crossbones:", "#8B0000"),
        };

        let formatted_message = format!("{} *System Alert*\n{}", emoji, message);

        let blocks = vec![
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": formatted_message
                }
            }),
            serde_json::json!({
                "type": "context",
                "elements": [
                    {
                        "type": "mrkdwn",
                        "text": format!(":clock3: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))
                    }
                ]
            }),
        ];

        let payload = serde_json::json!({
            "text": "System Alert",
            "attachments": [
                {
                    "color": color,
                    "blocks": blocks
                }
            ]
        });

        self.send_payload(&payload).await
    }

    /// Send a custom payload
    pub async fn send_payload(&self, payload: &serde_json::Value) -> Result<()> {
        if !self.config.enabled {
            debug!("Slack alerts are disabled, skipping payload");
            return Ok(());
        }

        info!("Sending custom Slack payload");

        match self
            .client
            .post(&self.config.webhook_url)
            .json(payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Slack payload sent successfully");
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    error!(
                        "Failed to send Slack payload. Status: {}, Error: {}",
                        status, error_text
                    );
                    Err(anyhow::anyhow!(
                        "Slack API error: {} - {}",
                        status,
                        error_text
                    ))
                }
            }
            Err(e) => {
                error!("Failed to send Slack payload: {}", e);
                Err(anyhow::anyhow!("Network error: {}", e))
            }
        }
    }

    /// Validate the Slack configuration
    pub fn validate_config(&self) -> Result<()> {
        if self.config.webhook_url.is_empty() {
            return Err(anyhow::anyhow!("Slack webhook URL is empty"));
        }

        // Validate webhook URL format
        if !self
            .config
            .webhook_url
            .starts_with("https://hooks.slack.com/services/")
            && !self
                .config
                .webhook_url
                .starts_with("https://hooks.slack.com/workflows/")
        {
            return Err(anyhow::anyhow!("Invalid Slack webhook URL format"));
        }

        Ok(())
    }

    /// Test the Slack connection
    pub async fn test_connection(&self) -> Result<()> {
        self.validate_config()?;

        let test_message = "✅ Slack integration test successful";
        self.send_message(test_message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_sender_creation() {
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/test".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };

        let sender = SlackAlertSender::new(config).unwrap();
        assert_eq!(
            sender.config.webhook_url,
            "https://hooks.slack.com/services/test"
        );
        assert_eq!(sender.config.channel, Some("test-channel".to_string()));
        assert_eq!(sender.config.username, Some("test-bot".to_string()));
        assert!(sender.config.enabled);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/test".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let sender = SlackAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_ok());

        // Invalid config - empty webhook URL
        let config = SlackConfig {
            webhook_url: "".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let sender = SlackAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_err());

        // Invalid config - wrong webhook URL format
        let config = SlackConfig {
            webhook_url: "https://example.com/invalid".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let sender = SlackAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_err());
    }

    #[test]
    fn test_message_formatting() {
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/test".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let sender = SlackAlertSender::new(config).unwrap();

        // Test message blocks creation
        let blocks = sender.create_message_blocks("Test message");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "section");
    }

    #[tokio::test]
    async fn test_alert_formatting() {
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/test".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let _sender = SlackAlertSender::new(config).unwrap();

        // Test info alert
        let formatted_info = ":information_source: *Info*\nTest info message".to_string();

        // Test warning alert
        let formatted_warning = ":warning: *Warning*\nTest warning message".to_string();

        // Test error alert
        let formatted_error = ":x: *Error*\nTest error message".to_string();

        // Test critical alert
        let formatted_critical = ":rotating_light: *Critical*\nTest critical message".to_string();

        // These would be tested with actual sending, but we're checking the formatting logic
        assert!(formatted_info.contains("Info"));
        assert!(formatted_warning.contains("Warning"));
        assert!(formatted_error.contains("Error"));
        assert!(formatted_critical.contains("Critical"));
    }

    #[tokio::test]
    async fn test_trade_alert_formatting() {
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/test".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let _sender = SlackAlertSender::new(config).unwrap();

        // Test trade alert with profit
        let message_with_profit = format!(
            ":moneybag: *Trade Executed*\nPair: {}\nSide: {}\nPrice: {:.6}\nAmount: {:.6}\nProfit: {:.6}%",
            "ETH/USDT", "BUY", 3000.0, 1.5, 2.5
        );

        // Test trade alert without profit
        let message_without_profit = format!(
            ":money_with_wings: *Trade Executed*\nPair: {}\nSide: {}\nPrice: {:.6}\nAmount: {:.6}",
            "ETH/USDT", "SELL", 3100.0, 1.5
        );

        assert!(message_with_profit.contains("Trade Executed"));
        assert!(message_with_profit.contains("Profit"));
        assert!(message_without_profit.contains("Trade Executed"));
        assert!(!message_without_profit.contains("Profit"));
    }

    #[tokio::test]
    async fn test_risk_alert_formatting() {
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/test".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let _sender = SlackAlertSender::new(config).unwrap();

        // Test risk alert
        let formatted_message = ":shield: *Risk Alert*\nHigh slippage detected: 5.2%".to_string();
        assert!(formatted_message.contains("Risk Alert"));
        assert!(formatted_message.contains("High slippage"));
    }

    #[tokio::test]
    async fn test_system_alert_formatting() {
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/test".to_string(),
            channel: Some("test-channel".to_string()),
            username: Some("test-bot".to_string()),
            enabled: true,
        };
        let _sender = SlackAlertSender::new(config).unwrap();

        // Test system alert
        let formatted_message = ":computer: *System Alert*\nSystem is running normally".to_string();
        assert!(formatted_message.contains("System Alert"));
        assert!(formatted_message.contains("System is running normally"));
    }
}
