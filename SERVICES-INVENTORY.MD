# Services Inventory

This document provides a comprehensive inventory of all services in the Sniper Bot trading system, categorized by type (web-based or terminal-based), with their ports and descriptions.

## Web-Based Services

These services expose HTTP APIs or web interfaces and can be accessed through a web browser or HTTP client.

### 1. Analytics Service
- **Crate**: svc-analytics
- **Port**: 3003
- **Framework**: Axum (Rust web framework)
- **Description**: Provides analytics data for trades and portfolio performance
- **API Endpoints**:
  - `GET /analytics/trades` - Get recent trades
  - `GET /analytics/portfolio` - Get portfolio analytics
  - `GET /analytics/performance` - Get performance metrics
  - `POST /analytics/record/trade` - Record a trade
  - `POST /analytics/record/portfolio` - Record portfolio data

### 2. Backtest Service
- **Crate**: svc-backtest
- **Port**: 3004
- **Framework**: Axum (Rust web framework)
- **Description**: Provides backtesting capabilities for trading strategies
- **API Endpoints**:
  - `POST /backtest/run` - Run a backtest
  - `POST /backtest/walk-forward` - Run walk-forward optimization
  - `GET /backtest/config` - Get backtest configuration
  - `PUT /backtest/config` - Update backtest configuration

### 3. Traditional Dashboard Service
- **Crate**: svc-dashboard
- **Port**: 3005
- **Framework**: Axum (Rust web framework)
- **Description**: Serves a traditional server-rendered dashboard with static files
- **API Endpoints**:
  - `GET /api/metrics` - Get system metrics
  - `GET /api/trades` - Get recent trades
  - `GET /api/portfolio` - Get portfolio data
  - `GET /api/performance` - Get performance metrics

### 4. WASM Dashboard Service
- **Crate**: svc-wasm-dashboard
- **Port**: 3006
- **Framework**: Axum (Rust web framework)
- **Description**: Serves the WebAssembly-based dashboard frontend
- **Endpoints**:
  - `GET /` - Service information
  - `GET /dashboard` - Main dashboard page
  - `GET /dashboard/*` - Static WASM dashboard assets

### 5. Gateway Service
- **Crate**: svc-gateway
- **Port**: Not specified (typically would be a low port like 80/443)
- **Framework**: None (appears to be a message bus service)
- **Description**: Acts as an entry point for external requests and handles mTLS

## Terminal-Based Services

These services run in the terminal without exposing web interfaces. They typically communicate through message buses or other inter-process communication mechanisms.

### 1. CEX Service
- **Crate**: svc-cex
- **Framework**: None (terminal-based)
- **Description**: Centralized exchange integration service

### 2. Executor Service
- **Crate**: svc-executor
- **Framework**: None (terminal-based)
- **Description**: Trade execution service across different venues

### 3. NFT Service
- **Crate**: svc-nft
- **Framework**: None (terminal-based)
- **Description**: NFT marketplace operations service

### 4. Policy Service
- **Crate**: svc-policy
- **Framework**: None (terminal-based)
- **Description**: Policy enforcement and user context management

### 5. Risk Service
- **Crate**: svc-risk
- **Framework**: None (terminal-based)
- **Description**: Real-time risk management service

### 6. Signals Service
- **Crate**: svc-signals
- **Framework**: None (terminal-based)
- **Description**: Signal processing service

### 7. Storage Service
- **Crate**: svc-storage
- **Framework**: None (terminal-based)
- **Description**: Database access and storage management

### 8. Strategy Service
- **Crate**: svc-strategy
- **Framework**: None (terminal-based)
- **Description**: Signal processing and strategy execution

## Shared Libraries (sniper-*)

These are not services but shared libraries used by the services:

- **sniper-amm**: AMM implementations (CPMM, Stableswap, Uniswap V3)
- **sniper-analytics**: Analytics and performance metrics
- **sniper-authz**: Authorization and authentication
- **sniper-backtest**: Backtesting framework
- **sniper-bench**: Performance benchmarks
- **sniper-cex**: Centralized exchange clients
- **sniper-chain**: Chain-specific implementations
- **sniper-core**: Core data structures and traits
- **sniper-exec**: Execution-related components
- **sniper-exit**: Exit strategies and mechanisms
- **sniper-keys**: Key management and cryptography
- **sniper-ml**: Machine learning models
- **sniper-nft**: NFT marketplace operations
- **sniper-policy**: Compliance and policy enforcement
- **sniper-risk**: Risk evaluation and honeypot detection
- **sniper-security**: Security controls and validation
- **sniper-signals**: Signal processing components
- **sniper-sim**: Simulation and testing utilities
- **sniper-storage**: Database access and Redis-based locks
- **sniper-telemetry**: Monitoring, metrics, and alerting
- **sniper-wasm-dashboard**: WebAssembly dashboard frontend

## Port Allocation Scheme

Services are assigned sequential port numbers:
- Analytics Service: 3003
- Backtest Service: 3004
- Traditional Dashboard Service: 3005
- WASM Dashboard Service: 3006
- Other services: Use message bus (NATS/Kafka) for communication rather than HTTP ports

## Service Communication

Services communicate through:
1. **HTTP APIs**: Web-based services expose RESTful APIs
2. **Message Bus**: NATS/Kafka for high-throughput communication between services
3. **gRPC**: For low-latency inter-service communication
4. **Shared Libraries**: Common functionality shared across services

## Running Services

To run all web-based services:
```bash
# Terminal 1: Analytics Service
cargo run --bin svc-analytics

# Terminal 2: Backtest Service
cargo run --bin svc-backtest

# Terminal 3: Traditional Dashboard Service
cargo run --bin svc-dashboard

# Terminal 4: WASM Dashboard Service
cargo run --bin svc-wasm-dashboard
```

Access the WASM dashboard at: http://localhost:3006/dashboard/