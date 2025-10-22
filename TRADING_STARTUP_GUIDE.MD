# Trading Startup Guide

This guide provides step-by-step instructions for configuring and starting the snipping bot for actual trading operations.

## Prerequisites

Before starting, ensure that:
1. All services are running (via Docker Compose or PowerShell scripts)
2. You have access to the configuration files in the `configs/` directory
3. You have your wallet keys ready for secure storage

## Step-by-Step Trading Setup

### 1. Configure Your Wallet Keys in Secure Key Storage

The bot uses a secure key management system to protect your wallet credentials.

#### Option A: Using Environment Variables (Development)
```bash
# Set your wallet private key as an environment variable
export WALLET_PRIVATE_KEY="your_private_key_here"
```

#### Option B: Using .env File (Development)
Create or update the `.env` file in the project root:
```env
WALLET_PRIVATE_KEY=your_private_key_here
```

#### Option C: Using Vault Integration (Production - Recommended)
For production environments, configure the `sniper-keys` crate to use HashiCorp Vault:
1. Update `configs/secrets.toml` with your Vault configuration
2. Ensure Vault is properly configured and accessible
3. Store your keys in Vault rather than in plain text files

**Security Note**: Never commit wallet keys to version control. The `.env.example` file shows the required format without exposing actual keys.

### 2. Set Up Your Trading Strategies in configs/strategies/

The bot supports multiple trading strategies that can be configured in the `configs/strategies/` directory.

#### Example Strategy Configuration
Create a new file in `configs/strategies/` (e.g., `my_strategy.toml`):

```toml
# Strategy Configuration
[strategy]
name = "My Trading Strategy"
enabled = true
description = "Custom strategy for token sniping"

# Trading Parameters
[strategy.trading]
chains = ["ethereum", "bsc", "polygon"]
max_concurrent_trades = 5
position_size_percent = 2.5
min_liquidity_usd = 10000

# Signal Processing
[strategy.signals]
enable_pair_created = true
enable_liquidity_added = true
enable_price_threshold = true

# Price Threshold Settings
[strategy.price_threshold]
enabled = true
threshold_percent = 300.0
time_window_minutes = 5

# Sniping Parameters
[strategy.sniping]
delay_ms = 100
gas_price_multiplier = 1.2
transaction_timeout_seconds = 30
```

#### Available Strategy Types
1. **Launch Sniping**: Detect new token launches and execute trades immediately
2. **Liquidity Addition**: Trade when liquidity is added to pools
3. **Price Threshold**: Execute trades when price changes exceed thresholds
4. **Arbitrage**: Exploit price differences across venues

### 3. Adjust Risk Parameters in configs/risk.toml

Risk management is critical for protecting your capital. Configure the risk parameters in `configs/risk.toml`:

```toml
# Risk Management Configuration
[risk]
enabled = true
max_daily_loss_percent = 5.0
max_position_loss_percent = 2.0
max_concurrent_positions = 10

# Honeypot Detection
[risk.honeypot]
enabled = true
check_ownership = true
check_mint_function = true
check_transfer_fees = true

# Correlation Risk
[risk.correlation]
enabled = true
max_correlation = 0.8
time_window_hours = 24

# Position Sizing
[risk.position_sizing]
max_position_size_percent = 5.0
min_position_size_usd = 100
risk_per_trade_percent = 1.0

# Stop Losses
[risk.stop_loss]
default_stop_loss_percent = 10.0
trailing_stop_enabled = true
trailing_stop_percent = 5.0

# Take Profit
[risk.take_profit]
default_take_profit_percent = 20.0
```

### 4. Monitor the Dashboard for Performance Metrics

The dashboard service provides real-time monitoring of your trading activities:

1. **Access the Dashboard**
   - Open your web browser
   - Navigate to http://localhost:3005

2. **Key Dashboard Features**
   - **Portfolio Overview**: Current portfolio value and asset allocation
   - **Trade History**: Recent trades and their performance
   - **Performance Metrics**: Win rate, profit/loss, Sharpe ratio
   - **Risk Metrics**: Current exposure and risk levels
   - **System Health**: Service status and resource utilization

3. **Alerts and Notifications**
   - Configure alerts in `configs/alerts.toml`
   - Set up email or Slack notifications for critical events
   - Monitor risk threshold breaches

### 5. Run Backtests to Validate Your Strategies Before Live Trading

Before going live, validate your strategies using the backtesting framework:

#### Running a Backtest via API
```bash
# Example backtest request
curl -X POST http://localhost:3004/backtest/run \
  -H "Content-Type: application/json" \
  -d '{
    "signals": [
      {
        "source": "dex",
        "kind": "pair_created",
        "chain": {
          "name": "ethereum",
          "id": 1
        },
        "token0": "0xTokenA",
        "token1": "0xWETH",
        "extra": {"pair": "0xPairAddress"},
        "seen_at_ms": 1500000
      }
    ]
  }'
```

#### Configuring Backtest Parameters
Update `configs/backtest.toml`:

```toml
[backtest]
enabled = true
start_time = 1640995200  # Unix timestamp
end_time = 1643673600    # Unix timestamp
initial_capital = 10000.0
trading_fee_percent = 0.003
slippage_percent = 0.005
max_position_size_percent = 0.1
```

#### Analyzing Backtest Results
The backtest service provides comprehensive metrics:
- Total profit/loss
- Win rate
- Sharpe ratio
- Maximum drawdown
- Individual trade performance

## Best Practices for Live Trading

### 1. Start Small
- Begin with small position sizes
- Gradually increase as you gain confidence
- Monitor performance closely

### 2. Continuous Monitoring
- Check the dashboard regularly
- Set up alerts for unusual activity
- Monitor market conditions

### 3. Risk Management
- Never risk more than you can afford to lose
- Use stop losses consistently
- Diversify across multiple strategies and assets

### 4. Regular Review
- Analyze trade performance weekly
- Adjust strategies based on results
- Stay updated with market trends

## Troubleshooting Common Issues

### 1. Connection Issues
- Verify all services are running
- Check network connectivity
- Ensure firewall settings allow required ports

### 2. Authentication Errors
- Verify wallet keys are correctly configured
- Check that keys have sufficient permissions
- Ensure keys are not expired or revoked

### 3. Performance Issues
- Monitor system resources
- Check service logs for errors
- Consider scaling services if needed

## Next Steps

Once you've completed the above setup:
1. Run initial backtests to validate your strategies
2. Start with paper trading (simulation mode)
3. Monitor performance for at least one week
4. Gradually transition to small live trades
5. Scale up as confidence and performance improve

For a comprehensive checklist to ensure all aspects of your trading setup are properly configured, refer to the [ENHANCED_TRADING_CHECKLIST.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/ENHANCED_TRADING_CHECKLIST.MD) which provides detailed steps for each phase of deployment.

To track your progress through the checklist, use the checklist tracking system described in [CHECKLIST_TRACKING_GUIDE.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/CHECKLIST_TRACKING_GUIDE.MD).

For detailed guidance on entering the market safely, refer to [ENTER-MARKET.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/ENTER-MARKET.MD) which provides a structured approach to market entry.

## Support and Resources

- Check the [DEVELOPMENT-GUIDE.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/DEVELOPMENT-GUIDE.MD) for detailed technical information
- Review [SECURITY_COMPLIANCE_REPORT.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/SECURITY_COMPLIANCE_REPORT.MD) for security best practices
- Refer to [PRODUCT_ROADMAP.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/PRODUCT_ROADMAP.MD) for upcoming features
- Use the [ENHANCED_TRADING_CHECKLIST.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/ENHANCED_TRADING_CHECKLIST.MD) for comprehensive deployment validation
- Track your progress with the [CHECKLIST_TRACKING_GUIDE.MD](file:///d%3A/auto-sync-daily-from-winC/snipping-bot/CHECKLIST_TRACKING_GUIDE.MD)
