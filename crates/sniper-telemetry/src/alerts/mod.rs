//! Alerting module for the sniper bot.
//!
//! This module provides functionality for sending alerts through various channels.

pub mod slack;
pub mod telegram;
pub mod webhook;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use slack::{SlackAlertSender, SlackConfig};
use webhook::{WebhookAlertSender, WebhookConfig};

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    /// Slack configuration
    pub slack_config: Option<SlackConfig>,
    /// Webhook configuration
    pub webhook_config: Option<WebhookConfig>,
    /// Enable/disable alert manager
    pub enabled: bool,
}

impl Default for AlertManagerConfig {
    fn default() -> Self {
        Self {
            slack_config: None,
            webhook_config: None,
            enabled: true,
        }
    }
}

/// Alert manager for sending alerts through multiple channels
pub struct AlertManager {
    slack_sender: Option<SlackAlertSender>,
    webhook_sender: Option<WebhookAlertSender>,
    config: AlertManagerConfig,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(config: AlertManagerConfig) -> Result<Self> {
        let slack_sender = if let Some(slack_config) = &config.slack_config {
            if slack_config.enabled {
                Some(SlackAlertSender::new(slack_config.clone())?)
            } else {
                None
            }
        } else {
            None
        };

        let webhook_sender = if let Some(webhook_config) = &config.webhook_config {
            if webhook_config.enabled {
                Some(WebhookAlertSender::new(webhook_config.clone())?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            slack_sender,
            webhook_sender,
            config,
        })
    }

    /// Send an alert through all configured channels
    pub async fn send_alert(&self, message: &str, severity: AlertSeverity) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let severity_clone = severity.clone();

        // Send to Slack if configured
        if let Some(slack_sender) = &self.slack_sender {
            if let Err(e) = slack_sender.send_alert(message, severity.clone()).await {
                eprintln!("Failed to send alert to Slack: {}", e);
            }
        }

        // Send to webhook if configured
        if let Some(webhook_sender) = &self.webhook_sender {
            if let Err(e) = webhook_sender.send_alert(message, severity_clone).await {
                eprintln!("Failed to send alert to webhook: {}", e);
            }
        }

        let severity_str = match &severity {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Error => "ERROR",
            AlertSeverity::Critical => "CRITICAL",
        };
        println!("ALERT [{}]: {}", severity_str, message);

        Ok(())
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
        if !self.config.enabled {
            return Ok(());
        }

        let message = if let Some(profit) = profit {
            format!(
                "Trade executed: {} {} @ {:.6} (Amount: {:.6}, Profit: {:.2}%)",
                side, pair, price, amount, profit
            )
        } else {
            format!(
                "Trade executed: {} {} @ {:.6} (Amount: {:.6})",
                side, pair, price, amount
            )
        };

        // Send to Slack if configured
        if let Some(slack_sender) = &self.slack_sender {
            if let Err(e) = slack_sender
                .send_trade_alert(pair, side, price, amount, profit)
                .await
            {
                eprintln!("Failed to send trade alert to Slack: {}", e);
            }
        }

        // Send to webhook if configured
        if let Some(webhook_sender) = &self.webhook_sender {
            if let Err(e) = webhook_sender
                .send_trade_alert(pair, side, price, amount, profit)
                .await
            {
                eprintln!("Failed to send trade alert to webhook: {}", e);
            }
        }

        println!("TRADE ALERT: {}", message);
        Ok(())
    }

    /// Send a risk alert
    pub async fn send_risk_alert(&self, message: &str, severity: AlertSeverity) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let severity_clone = severity.clone();

        // Send to Slack if configured
        if let Some(slack_sender) = &self.slack_sender {
            if let Err(e) = slack_sender
                .send_risk_alert(message, severity.clone())
                .await
            {
                eprintln!("Failed to send risk alert to Slack: {}", e);
            }
        }

        // Send to webhook if configured
        if let Some(webhook_sender) = &self.webhook_sender {
            if let Err(e) = webhook_sender
                .send_risk_alert(message, severity_clone)
                .await
            {
                eprintln!("Failed to send risk alert to webhook: {}", e);
            }
        }

        let severity_str = match &severity {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Error => "ERROR",
            AlertSeverity::Critical => "CRITICAL",
        };
        println!("RISK ALERT [{}]: {}", severity_str, message);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_manager_creation() {
        let config = AlertManagerConfig::default();
        let _manager = AlertManager::new(config);
        // Just testing that we can create an alert manager - no additional assertions needed
    }

    #[test]
    fn test_alert_manager_config() {
        let config = AlertManagerConfig {
            slack_config: Some(SlackConfig {
                webhook_url: "https://hooks.slack.com/services/test".to_string(),
                channel: Some("#test".to_string()),
                username: Some("test-bot".to_string()),
                enabled: true,
            }),
            webhook_config: Some(WebhookConfig {
                url: "https://webhook.example.com/alerts".to_string(),
                method: "POST".to_string(),
                headers: None,
                auth_token: None,
                auth_header: None,
                enabled: true,
                timeout_seconds: 30,
            }),
            enabled: true,
        };

        let manager = AlertManager::new(config).unwrap();
        assert!(manager.slack_sender.is_some());
        assert!(manager.webhook_sender.is_some());
    }
}
