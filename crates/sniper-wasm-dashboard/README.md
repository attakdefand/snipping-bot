# Sniper Bot WASM Dashboard

A WebAssembly-based dashboard for the Sniper Bot trading system, built with Rust and Yew.

## Features

- Real-time performance metrics visualization
- Trade history tracking
- Portfolio monitoring
- Responsive web interface
- Client-side rendering with WebAssembly

## Prerequisites

- Rust toolchain
- wasm-pack

## Building

### Using the build script

```bash
# On Unix-like systems
./build.sh

# On Windows
.\build.ps1
```

### Manual build

```bash
wasm-pack build --target web --out-dir pkg
```

## Running

To serve the dashboard locally:

```bash
# Using Python's built-in server (requires Python 3)
python -m http.server 8080

# Using Node.js http-server (requires npm install -g http-server)
http-server

# Using any other static file server
```

Then open http://localhost:8080 in your browser.

## Architecture

The dashboard is built using:

- **Rust** - Core logic and data processing
- **Yew** - Frontend framework for building web applications in Rust
- **WebAssembly** - Compilation target for running Rust in the browser
- **wasm-bindgen** - Bridge between Rust and JavaScript

## Components

1. **Dashboard** - Main overview with key metrics
2. **Trades** - Detailed trade history
3. **Portfolio** - Current portfolio status
4. **Settings** - Configuration options

## API Integration

The dashboard communicates with the backend services through REST APIs:

- `/api/metrics` - System metrics
- `/api/trades` - Recent trades
- `/api/portfolio` - Portfolio information
- `/api/performance` - Performance metrics

## Deployment

The WASM dashboard can be deployed by:

1. Building the package with `wasm-pack`
2. Copying the generated files to a web server
3. Ensuring the backend services are accessible

## Development

To develop the dashboard:

1. Make changes to the Rust code in `src/lib.rs`
2. Rebuild with `wasm-pack build`
3. Refresh the browser to see changes

## License

Apache 2.0