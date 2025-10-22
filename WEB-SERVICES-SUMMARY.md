# Web Services in the Snipping Bot Project

This document provides an overview of all web-based services in the snipping bot project and explains how to set up the complete WebAssembly-based dashboard.

## Existing Web-Based Services

### 1. svc-dashboard (Port 3005)
- **Type**: Traditional server-rendered dashboard
- **Technology**: Axum (Rust web framework) with static HTML/CSS/JavaScript
- **Features**:
  - Real-time performance metrics visualization
  - Trade history tracking
  - Portfolio monitoring
  - System metrics display
- **Implementation**: Uses server-side rendering with JavaScript enhancements
- **Access**: http://localhost:3005

### 2. svc-analytics (Port 3003)
- **Type**: API service
- **Technology**: Axum (Rust web framework)
- **Features**:
  - Analytics data API endpoints
  - Performance metrics calculation
  - Trade analysis services
- **Implementation**: Pure API service with JSON responses
- **Access**: http://localhost:3003/api/...

### 3. svc-backtest (Port 3004)
- **Type**: API service
- **Technology**: Axum (Rust web framework)
- **Features**:
  - Backtesting execution endpoints
  - Configuration management
  - Walk-forward optimization
- **Implementation**: Pure API service with JSON responses
- **Access**: http://localhost:3004/backtest/...

## New WASM-Based Services

### 1. sniper-wasm-dashboard (Library Crate)
- **Type**: WebAssembly frontend library
- **Technology**: Rust + Yew + WebAssembly
- **Features**:
  - Client-side rendering with WebAssembly
  - Real-time updates without page refresh
  - Modular component architecture
  - Type-safe development with Rust
- **Implementation**: Compiled to WebAssembly using wasm-pack

### 2. svc-wasm-dashboard (Service Crate)
- **Type**: Static file server for WASM dashboard
- **Technology**: Axum with CORS support
- **Features**:
  - Serves compiled WASM files
  - Handles CORS for client-side API calls
  - Lightweight static file serving
- **Access**: http://localhost:3006/dashboard

## Terminal-Based Services

The following services are terminal/command-line based and do not have web interfaces:

1. **svc-gateway** - Network gateway
2. **svc-signals** - Signal processing
3. **svc-strategy** - Trading strategy execution
4. **svc-executor** - Trade execution
5. **svc-risk** - Risk management
6. **svc-nft** - NFT operations
7. **svc-cex** - Centralized exchange integration
8. **svc-policy** - Policy enforcement
9. **svc-storage** - Data storage

## Setting Up the WASM Dashboard

### Prerequisites

1. Rust toolchain installed
2. wasm-pack installed (will be installed automatically by build scripts)

### Building the WASM Dashboard

```bash
# Navigate to the WASM dashboard crate
cd crates/sniper-wasm-dashboard

# On Unix-like systems
./build.sh

# On Windows
.\build.ps1
```

### Running the WASM Dashboard Service

```bash
# Start all services including the WASM dashboard
cargo run --bin svc-wasm-dashboard
```

Or use the project's service orchestration:

```powershell
# In PowerShell
.\scripts\run-all-services.ps1
```

### Accessing the Dashboard

Once running, the WASM dashboard will be available at:
http://localhost:3006/dashboard

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌────────────────────┐
│   WASM Frontend │◄──►│  WASM Dashboard  │◄──►│  Backend Services  │
│  (Client-side)  │    │   Service (3006) │    │ (3003, 3004, 3005) │
└─────────────────┘    └──────────────────┘    └────────────────────┘
                              ▲
                              │
                    ┌─────────┴──────────┐
                    │  Traditional Dash  │
                    │    (svc-dashboard  │
                    │        3005)       │
                    └────────────────────┘
```

## Benefits of the WASM Approach

1. **Performance**: Near-native performance in the browser
2. **Type Safety**: Rust's type system prevents many runtime errors
3. **Code Reuse**: Share data structures and logic between frontend and backend
4. **Security**: No JavaScript runtime vulnerabilities
5. **Developer Experience**: Single language (Rust) for entire stack

## Development Workflow

1. Modify Rust code in `crates/sniper-wasm-dashboard/src/lib.rs`
2. Rebuild with `wasm-pack build`
3. Refresh browser to see changes
4. Backend API calls automatically use existing service endpoints

## Future Enhancements

1. Add real-time WebSocket connections for live updates
2. Implement more sophisticated charting with WASM libraries
3. Add offline capabilities with WebAssembly storage
4. Implement progressive web app features