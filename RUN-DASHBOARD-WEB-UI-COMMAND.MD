# Run Dashboard Web UI Commands

This document contains the commands needed to run the complete web UI for the Sniper Bot trading system.

## Services Overview

The web UI consists of 4 services that need to be running:

1. **Analytics Service** (Port 3003) - Provides trade and portfolio data
2. **Backtest Service** (Port 3004) - Provides backtesting capabilities
3. **Traditional Dashboard Service** (Port 3005) - Traditional server-rendered dashboard
4. **WASM Dashboard Service** (Port 3006) - WebAssembly-based dashboard (main UI)

## Running the Services

### Option 1: Run each service in separate terminals

Open separate terminal windows for each service:

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

### Option 2: Using PowerShell (Windows)

```powershell
# Run all services in background
Start-Process -NoNewWindow cargo run --bin svc-analytics
Start-Process -NoNewWindow cargo run --bin svc-backtest
Start-Process -NoNewWindow cargo run --bin svc-dashboard
Start-Process -NoNewWindow cargo run --bin svc-wasm-dashboard
```

## Accessing the Web UI

Once all services are running, you can access the WebAssembly dashboard at:

- **WASM Dashboard**: http://localhost:3006/dashboard/

You can also access the other services directly:

- **Analytics Service API**: http://localhost:3003/
- **Backtest Service API**: http://localhost:3004/
- **Traditional Dashboard**: http://localhost:3005/

## Building the WASM Dashboard

If you make changes to the WASM dashboard code, you need to rebuild it:

```bash
# Build the WASM dashboard and copy assets to service directory
cd crates/svc-wasm-dashboard
powershell -ExecutionPolicy Bypass -File build.ps1
```

## Stopping the Services

To stop the services, you can:

1. Press Ctrl+C in each terminal window
2. Or kill the processes using Task Manager or command line:

```bash
taskkill /F /IM svc-analytics.exe
taskkill /F /IM svc-backtest.exe
taskkill /F /IM svc-dashboard.exe
taskkill /F /IM svc-wasm-dashboard.exe
```