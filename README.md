TwelveData API
=====

> **⚠️ DEPRECATED**: This crate is deprecated in favor of [twelve-data-client](https://crates.io/crates/twelve-data-client), which is auto-generated from the official Twelve Data OpenAPI specification and provides comprehensive API coverage with automatic builder pattern generation. Note: CSV format support is not yet available in the new client. If you require CSV responses, continue using this crate.

This is a Rust client for the https://twelvedata.com API.

**Fork Notice**: This is a fork of [metlos/twelve_data](https://github.com/metlos/twelve_data) with additional features:
- CSV response format support with configurable delimiters
- Generic `DataResponse<T>` wrapper for unified JSON/CSV handling
- Optional Axum integration for direct response conversion
- Case-insensitive format parameter handling

At this moment in time, it is by no means complete, only a couple of APIs useful for getting the stock price is implemented.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
twelve_data_inav = "0.5"
reqwest = "0.13.1"
tokio = { version = "1", features = ["full"] }
```

### Optional Features

Enable the `axum` feature for web framework integration:

```toml
[dependencies]
twelve_data_inav = { version = "0.5", features = ["axum"] }
```

## Quick Start

### JSON Response (default)

```rust
use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("TWELVE_DATA_API_KEY")?;
    let client = TwelveData::new(&api_key, Box::new(reqwest::Client::new()));

    // Request time series data
    let request = TimeSeriesRequest::builder()
        .symbol("AAPL".into())
        .interval(Interval::Day)
        .output_size(10)
        .build()?;

    let response = client.time_series(request).await?;
    let data = response.parse()?;

    println!("Symbol: {}", data.meta.unwrap().symbol);
    for quote in data.values {
        println!("Date: {}, Close: ${}", quote.datetime, quote.close);
    }

    Ok(())
}
```

### CSV Response

```rust
use twelve_data_inav::{
    TwelveData, 
    core::TimeSeriesRequest, 
    Interval, 
    OutputFormat, 
    CommonQueryParameters
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("TWELVE_DATA_API_KEY")?;
    let client = TwelveData::new(&api_key, Box::new(reqwest::Client::new()));

    // Request CSV format
    let request = TimeSeriesRequest::builder()
        .symbol("AAPL".into())
        .interval(Interval::Day)
        .output_size(5)
        .common(
            CommonQueryParameters::builder()
                .format(OutputFormat::CSV)
                .build()?
        )
        .build()?;

    let response = client.time_series(request).await?;
    
    if response.is_csv() {
        // Option 1: Parse CSV to structs
        let data = response.parse_csv()?;
        println!("Got {} values", data.values.len());
        
        // Option 2: Get raw CSV string
        // let csv_text = response.raw();
    }

    Ok(())
}
```

### Getting Current Quote

```rust
use twelve_data_inav::{TwelveData, core::QuoteRequest, Interval};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("TWELVE_DATA_API_KEY")?;
    let client = TwelveData::new(&api_key, Box::new(reqwest::Client::new()));

    let request = QuoteRequest::builder()
        .symbol("TSLA".into())
        .interval(Interval::Day)
        .build()?;

    let response = client.quote(request).await?;
    let quote = response.parse()?;

    println!("Symbol: {}", quote.symbol);
    println!("Current Price: ${}", quote.close);
    println!("Change: ${} ({}%)", quote.change, quote.percent_change);

    Ok(())
}
```

## Features

- **Multiple Response Formats**: JSON (default) and CSV with semicolon delimiter
- **Type-Safe Builders**: Ergonomic request builders for all API endpoints
- **Format Detection**: Automatic detection and parsing of response format
- **Axum Integration**: Optional `axum` feature for web framework integration
- **Case-Insensitive**: Format parameters accept `csv`, `CSV`, `json`, `JSON`, etc.

## Available Endpoints

- `time_series()` - Historical OHLCV data
- `quote()` - Real-time quote data
- `price()` - Current price
- `logo()` - Company logo URL

## Documentation

For detailed API documentation, run:
```bash
cargo doc --open
```

See [CSV_USAGE.md](CSV_USAGE.md) for examples of working with CSV responses.
