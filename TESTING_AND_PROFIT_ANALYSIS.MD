# Testing and Profit Analysis Guide

This guide explains how to test the snipping bot and analyze profit results.

## Prerequisites

Before running tests, ensure that:
1. Docker is installed and running
2. Rust 1.80+ is installed
3. All services are built (run `cargo build --release`)

## Running Tests

### 1. Start Required Services

The bot requires several services to be running for testing:

```bash
# Start the backtest service
cargo run --bin svc-backtest

# In a separate terminal, start the analytics service
cargo run --bin svc-analytics
```

Alternatively, you can use the Docker Compose setup:
```bash
docker-compose -f infra/compose-full.yml up -d
```

### 2. Run Backtests

Backtesting is the primary way to test strategies and see potential profits.

#### Using the PowerShell Script
```bash
# Run a backtest with the provided script
./scripts/run-backtest.ps1
```

#### Manual Backtest
```bash
# Send test signals to the backtest service
curl -X POST http://localhost:3004/backtest/run \
  -H "Content-Type: application/json" \
  -d @test_signals.json
```

### 3. Analyze Results

#### Backtest Results
The backtest service returns detailed results including:
- Total profit/loss
- Win rate
- Average profit per trade
- Risk metrics (drawdown, Sharpe ratio, etc.)
- Individual trade details

#### Performance Metrics
The analytics service provides real-time performance metrics:
- Trade execution status
- Portfolio value tracking
- Strategy performance indicators

## Understanding Profit Results

### Key Metrics to Monitor

1. **Total Profit/Loss**: The absolute dollar amount gained or lost
2. **Win Rate**: Percentage of profitable trades
3. **Average Profit Per Trade**: Average return per trade
4. **Maximum Drawdown**: Largest peak-to-trough decline
5. **Sharpe Ratio**: Risk-adjusted return measure

### Interpreting Results

#### Profitable Backtest
- Positive total profit/loss
- Win rate above 50%
- Consistent average profit per trade
- Manageable drawdown levels

#### Unprofitable Backtest
- Negative total profit/loss
- Low win rate
- High drawdown
- Poor risk-adjusted returns

## Testing Different Scenarios

### 1. Strategy Testing
Test different strategies by modifying the signal data sent to the backtest service.

### 2. Risk Parameter Testing
Adjust risk parameters in `configs/risk.toml` and re-run backtests.

### 3. Market Condition Testing
Use different historical data sets to test performance under various market conditions.

## Automated Testing Scripts

### Run Backtest Script
```bash
./scripts/run-backtest.ps1
```
This script:
1. Verifies the backtest service is running
2. Runs a backtest with sample signals
3. Displays key results
4. Saves full results to JSON file

### Check Analytics Script
```bash
./scripts/check-analytics.ps1
```
This script:
1. Verifies the analytics service is running
2. Fetches performance metrics
3. Displays key analytics data

## Best Practices for Testing

### 1. Start Simple
- Begin with basic strategies
- Use simple risk parameters
- Test with small data sets

### 2. Gradually Increase Complexity
- Add more sophisticated strategies
- Implement advanced risk controls
- Test with larger data sets

### 3. Document Results
- Save backtest results
- Track performance over time
- Compare different configurations

### 4. Validate Results
- Run multiple backtests
- Test with different data sets
- Cross-validate with other tools

## Next Steps

1. **Run multiple backtests** with different strategies
2. **Adjust risk parameters** to optimize performance
3. **Test with real historical data** for more accurate results
4. **Monitor live performance** when ready to trade
5. **Continuously refine strategies** based on results

## Troubleshooting

### Common Issues

1. **Services not starting**
   - Check that ports are not in use
   - Verify Docker is running (if using Docker)
   - Ensure all dependencies are installed

2. **Backtest returns no trades**
   - Check signal data format
   - Verify time ranges in configuration
   - Confirm strategy is enabled

3. **Poor performance results**
   - Review risk parameters
   - Check strategy configuration
   - Consider market conditions

### Getting Help

- Check the logs for error messages
- Review configuration files
- Consult the development guide
- Open an issue on GitHub if needed

## Conclusion

Testing is crucial for successful algorithmic trading. By following this guide, you can:
- Run comprehensive backtests
- Analyze profit potential
- Optimize strategies and risk parameters
- Make informed decisions about live trading

Remember to always test thoroughly before going live, and continuously monitor and adjust your strategies based on performance results.