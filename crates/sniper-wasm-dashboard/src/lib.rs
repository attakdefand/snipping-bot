//! WASM-based dashboard for the Sniper Bot
//!
//! This module provides a WebAssembly frontend for visualizing the sniper bot's
//! performance metrics, trades, and system status in real-time.

use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use serde::{Deserialize, Serialize};

/// App routes
#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/trades")]
    Trades,
    #[at("/portfolio")]
    Portfolio,
    #[at("/settings")]
    Settings,
}

/// Main application component
#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="app">
                <nav class="navbar">
                    <ul>
                        <li><Link<Route> to={Route::Home}>{"Dashboard"}</Link<Route>></li>
                        <li><Link<Route> to={Route::Trades}>{"Trades"}</Link<Route>></li>
                        <li><Link<Route> to={Route::Portfolio}>{"Portfolio"}</Link<Route>></li>
                        <li><Link<Route> to={Route::Settings}>{"Settings"}</Link<Route>></li>
                    </ul>
                </nav>
                
                <main>
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}

/// Route switcher
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Dashboard /> },
        Route::Trades => html! { <Trades /> },
        Route::Portfolio => html! { <Portfolio /> },
        Route::Settings => html! { <Settings /> },
    }
}

/// Dashboard component
#[function_component(Dashboard)]
fn dashboard() -> Html {
    // State for metrics
    let metrics = use_state(|| Metrics::default());
    let trades = use_state(|| Vec::<TradeAnalytics>::new());
    
    // Fetch metrics on component mount
    {
        let metrics = metrics.clone();
        let trades = trades.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(fetched_metrics) = fetch_metrics().await {
                    metrics.set(fetched_metrics);
                }
                
                if let Ok(fetched_trades) = fetch_trades().await {
                    trades.set(fetched_trades);
                }
            });
            || ()
        });
    }
    
    html! {
        <div class="dashboard">
            <h1>{"Sniper Bot Dashboard"}</h1>
            
            <div class="metrics-grid">
                <MetricCard title="Successful Trades" value={metrics.successful_trades.to_string()} />
                <MetricCard title="Failed Trades" value={metrics.failed_trades.to_string()} />
                <MetricCard title="Win Rate" value={format!("{:.2}%", metrics.win_rate * 100.0)} />
                <MetricCard title="Avg Return" value={format!("{:.2}%", metrics.avg_return * 100.0)} />
            </div>
            
            <div class="recent-trades">
                <h2>{"Recent Trades"}</h2>
                <div class="trades-list">
                    {for trades.iter().take(5).map(|trade| {
                        html! {
                            <div class="trade-item">
                                <span class="trade-chain">{&trade.chain}</span>
                                <span class="trade-tokens">{&trade.token_in} {" → "} {&trade.token_out}</span>
                                <span class="trade-return" style={if trade.actual_return.unwrap_or(0.0) > 0.0 { "color: green;" } else { "color: red;" }}>
                                    {format!("{:.2}%", trade.actual_return.unwrap_or(0.0) * 100.0)}
                                </span>
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}

/// Trades component
#[function_component(Trades)]
fn trades() -> Html {
    let trades = use_state(|| Vec::<TradeAnalytics>::new());
    
    {
        let trades = trades.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(fetched_trades) = fetch_trades().await {
                    trades.set(fetched_trades);
                }
            });
            || ()
        });
    }
    
    html! {
        <div class="trades">
            <h1>{"Recent Trades"}</h1>
            <table class="trades-table">
                <thead>
                    <tr>
                        <th>{"Timestamp"}</th>
                        <th>{"Chain"}</th>
                        <th>{"Tokens"}</th>
                        <th>{"Amount"}</th>
                        <th>{"Predicted Return"}</th>
                        <th>{"Actual Return"}</th>
                        <th>{"Gas Used"}</th>
                    </tr>
                </thead>
                <tbody>
                    {for trades.iter().map(|trade| {
                        html! {
                            <tr>
                                <td>{trade.timestamp}</td>
                                <td>{&trade.chain}</td>
                                <td>{&trade.token_in} {" → "} {&trade.token_out}</td>
                                <td>{trade.amount_in}</td>
                                <td>{format!("{:.2}%", trade.predicted_return * 100.0)}</td>
                                <td style={if trade.actual_return.unwrap_or(0.0) > 0.0 { "color: green;" } else { "color: red;" }}>
                                    {format!("{:.2}%", trade.actual_return.unwrap_or(0.0) * 100.0)}
                                </td>
                                <td>{trade.gas_used}</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        </div>
    }
}

/// Portfolio component
#[function_component(Portfolio)]
fn portfolio() -> Html {
    let portfolio = use_state(|| Option::<PortfolioAnalytics>::None);
    
    {
        let portfolio = portfolio.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(fetched_portfolio) = fetch_portfolio().await {
                    portfolio.set(Some(fetched_portfolio));
                }
            });
            || ()
        });
    }
    
    html! {
        <div class="portfolio">
            <h1>{"Portfolio"}</h1>
            {
                if let Some(portfolio_data) = &*portfolio {
                    html! {
                        <>
                            <div class="portfolio-summary">
                                <div class="summary-item">
                                    <h3>{"Total Value"}</h3>
                                    <p>{format!("${:.2}", portfolio_data.total_value)}</p>
                                </div>
                                <div class="summary-item">
                                    <h3>{"Risk Exposure"}</h3>
                                    <p>{format!("{:.2}%", portfolio_data.risk_exposure * 100.0)}</p>
                                </div>
                                <div class="summary-item">
                                    <h3>{"Volatility"}</h3>
                                    <p>{format!("{:.2}%", portfolio_data.volatility * 100.0)}</p>
                                </div>
                            </div>
                            
                            <div class="asset-allocation">
                                <h2>{"Asset Allocation"}</h2>
                                <ul>
                                    {for portfolio_data.asset_allocation.iter().map(|(asset, allocation)| {
                                        html! {
                                            <li>
                                                <span>{asset}</span>
                                                <span>{format!("{:.2}%", allocation * 100.0)}</span>
                                            </li>
                                        }
                                    })}
                                </ul>
                            </div>
                        </>
                    }
                } else {
                    html! {
                        <p>{"Loading portfolio data..."}</p>
                    }
                }
            }
        </div>
    }
}

/// Settings component
#[function_component(Settings)]
fn settings() -> Html {
    let api_endpoint = use_state(|| "http://localhost:3003".to_string());
    
    let on_api_endpoint_change = {
        let api_endpoint = api_endpoint.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            api_endpoint.set(input.value());
        })
    };
    
    html! {
        <div class="settings">
            <h1>{"Settings"}</h1>
            <div class="settings-form">
                <div class="form-group">
                    <label for="api-endpoint">{"API Endpoint"}</label>
                    <input 
                        type="text" 
                        id="api-endpoint" 
                        value={(*api_endpoint).clone()} 
                        onchange={on_api_endpoint_change}
                    />
                </div>
                <button>{"Save Settings"}</button>
            </div>
        </div>
    }
}

/// Metric card component
#[derive(Properties, PartialEq)]
struct MetricCardProps {
    title: String,
    value: String,
}

#[function_component(MetricCard)]
fn metric_card(props: &MetricCardProps) -> Html {
    html! {
        <div class="metric-card">
            <h3>{&props.title}</h3>
            <div class="metric-value">{&props.value}</div>
        </div>
    }
}

/// Metrics data structure
#[derive(Default, Clone, PartialEq)]
struct Metrics {
    successful_trades: u32,
    failed_trades: u32,
    win_rate: f64,
    avg_return: f64,
}

// Data structures for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TradeAnalytics {
    plan_id: String,
    timestamp: u64,
    chain: String,
    token_in: String,
    token_out: String,
    amount_in: u64,
    predicted_return: f64,
    actual_return: Option<f64>,
    execution_latency_ms: u64,
    gas_used: u64,
    risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortfolioAnalytics {
    timestamp: u64,
    total_value: f64,
    asset_allocation: std::collections::HashMap<String, f64>,
    risk_exposure: f64,
    correlation_matrix: std::collections::HashMap<String, std::collections::HashMap<String, f64>>,
    sector_allocation: std::collections::HashMap<String, f64>,
    geographic_exposure: std::collections::HashMap<String, f64>,
    concentration_risk: f64,
    volatility: f64,
    value_at_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BacktestResults {
    config: BacktestConfig,
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    total_profit_loss: f64,
    total_fees_paid: f64,
    total_slippage_loss: f64,
    win_rate: f64,
    avg_profit_per_trade: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    sortino_ratio: f64,
    calmar_ratio: f64,
    max_consecutive_wins: usize,
    max_consecutive_losses: usize,
    individual_trades: Vec<TradeResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BacktestConfig {
    start_time: i64,
    end_time: i64,
    initial_capital: f64,
    trading_fee_pct: f64,
    slippage_pct: f64,
    max_position_size_pct: f64,
    enabled: bool,
    data_path: Option<String>,
    execution_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TradeResult {
    plan_id: String,
    entry_time: i64,
    exit_time: i64,
    entry_price: f64,
    exit_price: f64,
    amount_in: f64,
    amount_out: f64,
    profit_loss: f64,
    profit_loss_pct: f64,
    fees_paid: f64,
    slippage_loss: f64,
    position_size_pct: f64,
    execution_details: ExecutionDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionDetails {
    model_used: String,
    queue_position: Option<usize>,
    partial_fills: Vec<PartialFill>,
    latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartialFill {
    price: f64,
    amount: f64,
    timestamp: u64,
}

/// Fetch metrics from the backend
async fn fetch_metrics() -> Result<Metrics, JsValue> {
    // Fetch trade analytics from the analytics service
    let trades_response = fetch_from_api("http://localhost:3003/analytics/trades").await?;
    let trades: Vec<TradeAnalytics> = serde_json::from_str(&trades_response).map_err(|e| JsValue::from_str(&format!("Failed to parse trades: {:?}", e)))?;
    
    // Calculate metrics from trade data
    let successful_trades = trades.iter().filter(|trade| trade.actual_return.unwrap_or(0.0) > 0.0).count() as u32;
    let failed_trades = trades.len() as u32 - successful_trades;
    
    let win_rate = if trades.len() > 0 {
        successful_trades as f64 / trades.len() as f64
    } else {
        0.0
    };
    
    let avg_return = if successful_trades > 0 {
        let total_return: f64 = trades.iter()
            .filter_map(|trade| trade.actual_return)
            .sum();
        total_return / successful_trades as f64
    } else {
        0.0
    };
    
    Ok(Metrics {
        successful_trades,
        failed_trades,
        win_rate,
        avg_return,
    })
}

/// Fetch trades from the backend
async fn fetch_trades() -> Result<Vec<TradeAnalytics>, JsValue> {
    let response = fetch_from_api("http://localhost:3003/analytics/trades").await?;
    let trades: Vec<TradeAnalytics> = serde_json::from_str(&response).map_err(|e| JsValue::from_str(&format!("Failed to parse trades: {:?}", e)))?;
    Ok(trades)
}

/// Fetch portfolio from the backend
async fn fetch_portfolio() -> Result<PortfolioAnalytics, JsValue> {
    let response = fetch_from_api("http://localhost:3003/analytics/portfolio").await?;
    let portfolio: PortfolioAnalytics = serde_json::from_str(&response).map_err(|e| JsValue::from_str(&format!("Failed to parse portfolio: {:?}", e)))?;
    Ok(portfolio)
}

/// Generic function to fetch data from an API endpoint
async fn fetch_from_api(url: &str) -> Result<String, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    
    let request = Request::new_with_str_and_init(url, &opts)?;
    
    let window = web_sys::window().ok_or("No global `window` exists")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Response is not a Response")?;
    
    let text = JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap_or_default())
}

/// Start the WASM application
#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<App>::new().render();
}