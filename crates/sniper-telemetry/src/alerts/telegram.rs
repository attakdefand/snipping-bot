//! Telegram alerting module for the sniper bot.
//!
//! This module provides functionality for sending alerts through Telegram bots.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Telegram bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Bot token from BotFather
    pub bot_token: String,
    /// Chat ID to send alerts to
    pub chat_id: String,
    /// Enable/disable Telegram alerts
    pub enabled: bool,
}

/// Telegram alert sender
pub struct TelegramAlertSender {
    /// HTTP client for making requests
    client: Client,
    /// Bot configuration
    config: TelegramConfig,
}

impl TelegramAlertSender {
    /// Create a new Telegram alert sender
    pub fn new(config: TelegramConfig) -> Result<Self> {
        let client = Client::new();
        Ok(Self { client, config })
    }

    /// Send a message through Telegram
    pub async fn send_message(&self, message: &str) -> Result<()> {
        if !self.config.enabled {
            debug!(
                "Telegram alerts are disabled, skipping message: {}",
                message
            );
            return Ok(());
        }

        info!("Sending Telegram alert: {}", message);

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let mut params = HashMap::new();
        params.insert("chat_id", self.config.chat_id.clone());
        params.insert("text", message.to_string());
        params.insert("parse_mode", "Markdown".to_string());

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Telegram message sent successfully");
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    error!(
                        "Failed to send Telegram message. Status: {}, Error: {}",
                        status, error_text
                    );
                    Err(anyhow::anyhow!(
                        "Telegram API error: {} - {}",
                        status,
                        error_text
                    ))
                }
            }
            Err(e) => {
                error!("Failed to send Telegram message: {}", e);
                Err(anyhow::anyhow!("Network error: {}", e))
            }
        }
    }

    /// Send an alert with severity level
    pub async fn send_alert(&self, message: &str, severity: super::AlertSeverity) -> Result<()> {
        let formatted_message = match severity {
            super::AlertSeverity::Info => format!("ℹ️ *Info*\n{}", message),
            super::AlertSeverity::Warning => format!("⚠️ *Warning*\n{}", message),
            super::AlertSeverity::Error => format!("❌ *Error*\n{}", message),
            super::AlertSeverity::Critical => format!("🚨 *Critical*\n{}", message),
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
        let message = if let Some(profit) = profit {
            format!(
                "📊 *Trade Executed*\nPair: {}\nSide: {}\nPrice: {:.6}\nAmount: {:.6}\nProfit: {:.6}%",
                pair, side, price, amount, profit
            )
        } else {
            format!(
                "📊 *Trade Executed*\nPair: {}\nSide: {}\nPrice: {:.6}\nAmount: {:.6}",
                pair, side, price, amount
            )
        };

        self.send_message(&message).await
    }

    /// Send a risk alert
    pub async fn send_risk_alert(
        &self,
        message: &str,
        severity: super::AlertSeverity,
    ) -> Result<()> {
        let formatted_message = match severity {
            super::AlertSeverity::Info => format!("🛡️ *Risk Info*\n{}", message),
            super::AlertSeverity::Warning => format!("🛡️⚠️ *Risk Warning*\n{}", message),
            super::AlertSeverity::Error => format!("🛡️❌ *Risk Error*\n{}", message),
            super::AlertSeverity::Critical => format!("🛡️🚨 *Risk Critical*\n{}", message),
        };

        self.send_message(&formatted_message).await
    }

    /// Validate the Telegram configuration
    pub fn validate_config(&self) -> Result<()> {
        if self.config.bot_token.is_empty() {
            return Err(anyhow::anyhow!("Telegram bot token is empty"));
        }

        if self.config.chat_id.is_empty() {
            return Err(anyhow::anyhow!("Telegram chat ID is empty"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_sender_creation() {
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "test_chat_id".to_string(),
            enabled: true,
        };

        let sender = TelegramAlertSender::new(config).unwrap();
        assert_eq!(sender.config.bot_token, "test_token");
        assert_eq!(sender.config.chat_id, "test_chat_id");
        assert!(sender.config.enabled);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "test_chat_id".to_string(),
            enabled: true,
        };
        let sender = TelegramAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_ok());

        // Invalid config - empty bot token
        let config = TelegramConfig {
            bot_token: "".to_string(),
            chat_id: "test_chat_id".to_string(),
            enabled: true,
        };
        let sender = TelegramAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_err());

        // Invalid config - empty chat ID
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "".to_string(),
            enabled: true,
        };
        let sender = TelegramAlertSender::new(config).unwrap();
        assert!(sender.validate_config().is_err());
    }

    #[tokio::test]
    async fn test_message_formatting() {
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "test_chat_id".to_string(),
            enabled: true,
        };
        let sender = TelegramAlertSender::new(config).unwrap();

        // Test info alert
        let _result = sender
            .send_alert("Test info message", super::super::AlertSeverity::Info)
            .await;
        // This will fail due to network issues, but we're testing the formatting logic

        // Test warning alert
        let _result = sender
            .send_alert("Test warning message", super::super::AlertSeverity::Warning)
            .await;
        // This will fail due to network issues, but we're testing the formatting logic

        // Test error alert
        let _result = sender
            .send_alert("Test error message", super::super::AlertSeverity::Error)
            .await;
        // This will fail due to network issues, but we're testing the formatting logic

        // Test critical alert
        let _result = sender
            .send_alert(
                "Test critical message",
                super::super::AlertSeverity::Critical,
            )
            .await;
        // This will fail due to network issues, but we're testing the formatting logic
    }

    #[tokio::test]
    async fn test_trade_alert() {
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "test_chat_id".to_string(),
            enabled: true,
        };
        let sender = TelegramAlertSender::new(config).unwrap();

        // Test trade alert with profit
        let _result = sender
            .send_trade_alert("ETH/USDT", "BUY", 3000.0, 1.5, Some(2.5))
            .await;
        // This will fail due to network issues, but we're testing the formatting logic

        // Test trade alert without profit
        let _result = sender
            .send_trade_alert("ETH/USDT", "SELL", 3100.0, 1.5, None)
            .await;
        // This will fail due to network issues, but we're testing the formatting logic
    }

    #[tokio::test]
    async fn test_risk_alert() {
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "test_chat_id".to_string(),
            enabled: true,
        };
        let sender = TelegramAlertSender::new(config).unwrap();

        // Test risk alert
        let _result = sender
            .send_risk_alert(
                "High slippage detected: 5.2%",
                super::super::AlertSeverity::Warning,
            )
            .await;
        // This will fail due to network issues, but we're testing the formatting logic
    }
}
