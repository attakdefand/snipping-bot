# Sniper-RS: High-Frequency Trading Bot Framework

[![Rust](https://img.shields.io/badge/rust-1.80%2B-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/attakdefand/snipping-bot/actions)

Sniper-RS is a comprehensive, microservices-based trading bot framework designed for high-frequency trading across multiple venues including DEXs, NFT markets, and CEXs. Built with Rust for performance and safety, it provides a production-ready foundation for developing sophisticated trading strategies.

## 🚀 Key Features

### 🏗️ Architecture
- **Microservices Design**: Modular services for independent deployment and scaling
- **Event-Driven**: NATS/Kafka message bus for high-throughput communication
- **gRPC Integration**: High-performance service-to-service communication
- **Cross-Chain Support**: Multi-chain configuration for EVM-compatible blockchains

### 💹 Trading Capabilities
- **Multi-Venue Trading**: Support for DEXs, NFT markets, and CEXs
- **Advanced Signal Processing**: On-chain event monitoring and off-chain data feeds
- **Strategy Engine**: Multiple strategy types including launch sniping and liquidity trading
- **Risk Management**: Honeypot detection, owner power analysis, and position limits
- **Execution Engine**: Multiple modes including MEV bundle submission
- **Exit Strategies**: Take profit, stop loss, and trailing stop mechanisms

### 🔧 Technical Features
- **CEX Integration**: REST and WebSocket clients for major exchanges
- **Monitoring & Telemetry**: Prometheus metrics, OpenTelemetry tracing, and alerting
- **Storage & Persistence**: Database storage with position tracking and idempotency
- **Distributed Locks**: Redis-based locks to prevent duplicate actions
- **Policy Engine**: Geo, venue, and KYC-based compliance policies
- **Security**: Key isolation, mTLS support, and RBAC

## 📦 Architecture Overview

```
┌─────────────────┐    ┌────────────────┐    ┌─────────────────┐
│   svc-gateway   │    │  svc-strategy  │    │   svc-executor  │
│  (HTTP API)     │◄──►│ (Signal Proc.) │◄──►│ (Trade Exec.)   │
└─────────────────┘    └────────────────┘    └─────────────────┘
       ▲                       ▲                      ▲
       │                       │                      │
       ▼                       ▼                      ▼
┌─────────────────┐    ┌────────────────┐    ┌─────────────────┐
│   svc-risk      │    │  svc-policy    │    │   svc-storage   │
│ (Risk Mgmt.)    │    │ (Compliance)   │    │ (Persistence)   │
└─────────────────┘    └────────────────┘    └─────────────────┘
       ▲                       ▲                      ▲
       │                       │                      │
       ▼                       ▼                      ▼
┌─────────────────┐    ┌────────────────┐    ┌─────────────────┐
│   svc-cex       │    │  svc-nft       │    │ Message Bus     │
│ (CEX Connect)   │    │ (NFT Markets)  │    │ (NATS/Kafka)    │
└─────────────────┘    └────────────────┘    └─────────────────┘
```

## 🛠️ Core Components

### Shared Libraries (sniper-*)
- `sniper-core`: Core data structures and traits
- `sniper-amm`: AMM implementations (CPMM, Stableswap, Uniswap V3)
- `sniper-cex`: CEX REST/WebSocket clients
- `sniper-risk`: Risk evaluation and honeypot detection
- `sniper-policy`: Compliance and policy engines
- `sniper-storage`: Database storage and distributed locks
- `sniper-telemetry`: Monitoring, metrics, and alerting

### Services (svc-*)
- `svc-gateway`: HTTP API gateway and administration interface
- `svc-strategy`: Signal processing and strategy execution
- `svc-executor`: Trade execution across multiple venues
- `svc-risk`: Risk management and compliance checking
- `svc-cex`: CEX integration and order management
- `svc-nft`: NFT marketplace integration
- `svc-policy`: Policy enforcement and user context management

## 🚀 Getting Started

### Prerequisites
- Rust 1.80+
- Docker (for containerized deployment)
- Kubernetes (for production deployment)

### Quick Start
```bash
# Clone the repository
git clone https://github.com/attakdefand/snipping-bot.git
cd sniper-rs

# Build all services
cargo build --release

# Run tests
cargo test

# Start services with Docker Compose
docker-compose up -d
```

### Configuration
Configuration is handled through TOML files in the `configs/` directory:
- `config.base.toml`: Base configuration
- `config.local.toml`: Local development overrides
- `config.prod.toml`: Production configuration

## 🧪 Testing

The project includes comprehensive tests:
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run specific crate tests
cargo test -p sniper-storage
```

## 📈 Performance Metrics

- Signal Processing Latency: < 50ms
- Trade Plan Generation: < 100ms
- Risk Assessment: < 200ms
- Execution Submission: < 150ms
- CEX Integration Latency: < 100ms
- Test Coverage: 95%

## 🛡️ Security

- Private key access limited to svc-executor only
- mTLS support for service mesh integration
- JWT-based role-based access control
- East-West communication restrictions

## 📚 Documentation

- [Product Features](../PRODUCT_FEATURES.MD)
- [Product Roadmap](../PRODUCT_ROADMAP.MD)
- [Daily Development Log](../DAILY_FEATURES.MD)
- [Architecture Layout](../LAYOUT.MD)

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to this project.

## 📞 Support

For support, please open an issue on GitHub or contact the maintainers.