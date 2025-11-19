# Next Backtest Implementation Roadmap

Based on the analysis of BACKTEST-FEATURES.MD and the current implementation, here are the remaining features that need to be implemented:

## 1. Data Layer
- **Parquet/CSV Market Adapters** - Implement comprehensive data adapters for different formats
- **Book/Tick Compactor** - Create functionality to compact market data
- **Resampler** - Implement data resampling capabilities
- **Corporate Action & Split Adjuster** - Handle corporate actions and stock splits
- **Time Alignment (Multi-venue Clock)** - Synchronize timestamps across different venues
- **Missing-Data Handlers** - Implement strategies for handling missing data points
- **Session Calendars** - Create trading session calendars for different markets

## 2. Microstructure & Execution Simulator
- **Full Order Types Support** - Implement IOC/FOK/PO, post-only, hidden/iceberg orders
- **Queue Position Modeling** - Enhance queue position simulation
- **Adverse Selection Protection** - Implement adverse selection protection mechanisms
- **Book-Walk Depth Caps** - Add depth constraints to order book walking
- **Funding Fees (Perps)** - Add support for perpetual funding fees
- **Borrow Fees** - Implement borrow fee calculations
- **Circuit Constraints** - Implement price bands, auction/open/close, downtime handling
- **TWAP Implementation** - Create Time-Weighted Average Price execution model

## 3. Risk Engine
- **Volatility Targeting** - Implement volatility-based position sizing
- **Kelly-Fraction Capped Position Sizing** - Add Kelly criterion with caps
- **Gamma Limits** - Implement gamma risk limits
- **Correlation Caps** - Add correlation-based risk constraints
- **Warm-up Windows** - Implement model warm-up periods
- **Feature Store** - Create feature storage and management system
- **Label Leakage Guards** - Implement protection against label leakage

## 4. Accounting & PnL
- **Mark-to-Last/Bid/Ask** - Add alternative marking methods
- **FIFO/LIFO Lots** - Implement inventory accounting methods
- **Multi-currency NAV with FX** - Handle multiple currencies and FX rates
- **Stablecoin Pegs** - Account for stablecoin depegging scenarios

## 5. Metrics & Reports
- **Heatmaps by Hour/Day/Venue** - Generate temporal and venue-based heatmaps
- **Alpha Attribution by Feature/Signal/Bucket** - Implement detailed alpha attribution
- **Stability Curves** - Create stability analysis visualizations
- **Sensitivity Curves** - Implement sensitivity analysis tools

## 6. Walk-Forward & Robustness Testing
- **K-fold Temporal CV** - Implement temporal cross-validation
- **Regime Split** - Add regime-based training/testing splits
- **Stress Windows** - Implement stress scenario testing
- **Hyperparameter Search** - Add grid/random/Bayesian/Optuna optimization
- **Early-Stopping vs Overfit Guards** - Implement overfitting protection
- **Bootstrap/Monte-Carlo** - Add resampling techniques
- **Alternative Fills Simulation** - Implement optimistic/median/pessimistic fill scenarios

## 7. Paper-Trade Backtest
- **Live Dashboards** - Create real-time monitoring dashboards
- **Latency Histograms** - Implement latency analysis tools
- **Expected vs Simulated Fills** - Add fill comparison analysis

## 8. On-chain Fork Testing
- **Mainnet-Fork Replay** - Implement actual mainnet forking capabilities
- **Gas Oracle Models** - Add gas price modeling
- **Pending-Tx Queue** - Implement pending transaction queue simulation
- **Reorg Tests** - Enhance reorg simulation capabilities
- **Router Path Finders** - Implement routing path analysis
- **CLAMM Tick-Cross Modeling** - Add concentrated liquidity modeling
- **Price Oracle Staleness** - Implement oracle staleness simulation

## 9. Portfolio / Multi-Venue Backtest
- **Smart-router Simulation** - Enhance smart routing capabilities
- **Inventory-aware MM Backtest** - Implement market-making inventory models
- **Venue Selection Policy** - Enhance venue selection algorithms
- **Cross-venue Latency Matrix** - Implement detailed latency modeling
- **Failover Behaviors** - Add comprehensive failover strategies

## 10. Beyond Backtests Features
- **Forward Testing Suite** - Implement Shadow/Canary testing modes
- **Kill-switches** - Enhance kill-switch functionality
- **Live Execution Controls** - Add throttle/ratelimit, dedup, idempotent order keys
- **Venue Health Scores** - Implement venue health monitoring
- **Automatic Venue Failover** - Add automatic failover mechanisms
- **Research & Experiment Platform** - Create experiment tracker, feature store, model registry
- **Scenario & Chaos Packs** - Enhance scenario testing capabilities
- **What-if Engine** - Implement what-if analysis engine
- **TCA & Capacity Analysis** - Enhance transaction cost and capacity analysis
- **Compliance & Audit** - Add deterministic seeds, reproducible runs, signed artifacts
- **Full Trade & Decision Logs** - Implement comprehensive logging
- **Policy Approvals** - Add policy approval workflows
- **Observability** - Enhance monitoring and observability features
- **Strategy Metrics** - Add signal drift, live-vs-backtest variance alarms
- **Post-trade Health** - Implement reject spike and cancel/replace loop detection
- **Data Ops** - Add data lineage, schema contracts, quality monitors
- **Synthetic Data Generators** - Create synthetic data generation capabilities

## Priority Implementation Order

1. **High Priority** (Core functionality needed for production use):
   - Feature Store
   - Enhanced Order Types Support
   - TWAP Implementation
   - Volatility Targeting
   - Heatmaps and Advanced Metrics
   - Live Dashboards

2. **Medium Priority** (Important but not critical for basic operation):
   - K-fold Temporal CV
   - Bootstrap/Monte-Carlo
   - Gas Oracle Models
   - Router Path Finders
   - What-if Engine
   - Data Ops Pipeline

3. **Low Priority** (Nice to have for advanced use cases):
   - Synthetic Data Generators
   - Full Compliance & Audit Features
   - Advanced Scenario Testing
   - Research & Experiment Platform

This roadmap should guide the implementation of the remaining backtest features to bring the system to full parity with the specification in BACKTEST-FEATURES.MD.