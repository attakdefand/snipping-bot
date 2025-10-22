# Complete Web UI Setup for Snipping Bot

This document provides a comprehensive guide to all web-based interfaces in your snipping bot project and how to set up the complete WebAssembly-based dashboard.

## Overview of Web-Based Services

Your snipping bot project includes multiple web-based services with different technologies and purposes:

### 1. Traditional Dashboard Service (svc-dashboard)
- **Port**: 3005
- **Technology**: Server-rendered HTML with JavaScript enhancements
- **Access**: http://localhost:3005
- **Features**: Real-time metrics, trade history, portfolio monitoring

### 2. Analytics API Service (svc-analytics)
- **Port**: 3003
- **Technology**: REST API endpoints
- **Access**: http://localhost:3003/api/...
- **Features**: Analytics data, performance metrics, trade analysis

### 3. Backtest API Service (svc-backtest)
- **Port**: 3004
- **Technology**: REST API endpoints
- **Access**: http://localhost:3004/backtest/...
- **Features**: Backtesting execution, configuration management

### 4. WASM Dashboard Service (svc-wasm-dashboard) - NEW
- **Port**: 3006
- **Technology**: WebAssembly client-side rendering
- **Access**: http://localhost:3006/dashboard
- **Features**: Modern UI with real-time updates, modular components

## Terminal-Based Services (No Web UI)

The following services operate in the terminal and do not have web interfaces:
- svc-gateway
- svc-signals
- svc-strategy
- svc-executor
- svc-risk
- svc-nft
- svc-cex
- svc-policy
- svc-storage

## Complete WASM Dashboard Setup

### Step 1: Install Prerequisites

Ensure you have the Rust toolchain installed:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Step 2: Build the WASM Dashboard

```bash
# Navigate to the project root
cd /path/to/snipping-bot

# Build the WASM components
.\build-wasm.ps1
```

This will:
1. Install wasm-pack if not already installed
2. Compile the Rust code to WebAssembly
3. Generate the necessary files in the `pkg/` directory

### Step 3: Run the WASM Dashboard Service

You can run the WASM dashboard service in several ways:

**Option 1: Run just the WASM dashboard service**
```bash
cargo run --bin svc-wasm-dashboard
```

**Option 2: Run all services including the WASM dashboard**
```bash
.\scripts\run-all-services.ps1
```

### Step 4: Access the Dashboard

Once the service is running, access the WASM dashboard at:
http://localhost:3006/dashboard

## WASM Dashboard Features

The new WASM-based dashboard provides:

1. **Client-Side Rendering**: Faster updates without page refreshes
2. **Real-Time Updates**: Live data streaming using WebAssembly
3. **Modular Components**: Reusable UI components built with Yew
4. **Type Safety**: Full Rust type safety in the browser
5. **Performance**: Near-native performance in the browser

### Components Included:

- **Dashboard**: Main overview with key metrics
- **Trades**: Detailed trade history
- **Portfolio**: Current portfolio status
- **Settings**: Configuration options

## API Integration

The WASM dashboard communicates with backend services through REST APIs:

- **Metrics**: http://localhost:3003/api/metrics
- **Trades**: http://localhost:3003/api/trades
- **Portfolio**: http://localhost:3003/api/portfolio
- **Performance**: http://localhost:3003/api/performance

## Development Workflow

To develop and modify the WASM dashboard:

1. Edit the Rust code in `crates/sniper-wasm-dashboard/src/lib.rs`
2. Rebuild with `wasm-pack build --target web --out-dir pkg`
3. Refresh the browser to see changes
4. Backend API calls automatically use existing service endpoints

## Architecture Benefits

### Traditional Dashboard (svc-dashboard)
- Simpler setup
- Server-rendered content
- Good for static content
- JavaScript enhancements for interactivity

### WASM Dashboard (svc-wasm-dashboard)
- Client-side rendering
- Near-native performance
- Type-safe development
- Shared code between frontend and backend
- No JavaScript runtime vulnerabilities

## Project Structure

```
snipping-bot/
├── crates/
│   ├── sniper-wasm-dashboard/     # WASM frontend library
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs             # Main WASM logic
│   │   ├── index.html             # HTML entry point
│   │   ├── build.sh               # Unix build script
│   │   ├── build.ps1              # Windows build script
│   │   └── README.md
│   └── svc-wasm-dashboard/        # WASM dashboard service
│       ├── Cargo.toml
│       └── src/main.rs            # Service entry point
├── build-wasm.ps1                 # Project-level build script
└── ...
```

## Port Allocation Scheme

Following the project's established port allocation:
- svc-analytics: 3003
- svc-backtest: 3004
- svc-dashboard: 3005
- svc-wasm-dashboard: 3006 (NEW)

## Troubleshooting

### Common Issues:

1. **wasm-pack not found**: Run the installation command provided in the build script
2. **CORS errors**: The service includes permissive CORS settings for development
3. **API connection issues**: Ensure backend services are running on their respective ports

### Verifying Services:

```bash
# Check if all services compile
cargo check --all

# Run specific service
cargo run --bin svc-wasm-dashboard

# Check service status
.\scripts\check-services.ps1
```

## Future Enhancements

Planned improvements for the WASM dashboard:

1. **WebSocket Integration**: Real-time data streaming
2. **Advanced Charting**: Interactive financial charts using WASM libraries
3. **Offline Support**: Local storage and offline capabilities
4. **PWA Features**: Progressive web app functionality
5. **Enhanced Security**: Authentication and authorization

## Conclusion

Your snipping bot now has a modern, high-performance web interface option using WebAssembly technology. This complements the existing traditional dashboard and provides users with a choice of interfaces based on their needs and preferences.

The WASM dashboard offers superior performance and type safety while maintaining compatibility with the existing backend services. Developers can leverage Rust's powerful type system and performance characteristics to build robust, maintainable frontend applications.