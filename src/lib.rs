//! # TwelveData API Client
//!
//! A Rust client library for the [Twelve Data](https://twelvedata.com) financial API.
//! Supports both JSON and CSV response formats with automatic parsing and type-safe handling.
//!
//! ## Features
//!
//! - **Multiple HTTP Clients**: Support for reqwest, surf, and wreq (via feature flags)
//! - **CSV Support**: Parse CSV responses with configurable delimiters (defaults to semicolon)
//! - **Unified Response Handling**: Generic `DataResponse<T>` works with both JSON and CSV
//! - **Axum Integration**: Optional `IntoResponse` trait implementation for web frameworks
//! - **Type Safety**: Strongly-typed request builders and response structures
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = TwelveData::new("your_api_key", Box::new(reqwest::Client::new()));
//!
//! let request = TimeSeriesRequest::builder()
//!     .symbol("AAPL".into())
//!     .interval(Interval::Day)
//!     .output_size(10)
//!     .build()?;
//!
//! let response = client.time_series(request).await?;
//! let data = response.parse()?; // Auto-detects JSON or CSV
//! # Ok(())
//! # }
//! ```
//!
//! ## CSV Support
//!
//! Request CSV format and parse the results:
//!
//! ```rust,no_run
//! use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval, OutputFormat, CommonQueryParameters};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
//! let request = TimeSeriesRequest::builder()
//!     .symbol("AAPL".into())
//!     .interval(Interval::Day)
//!     .common(
//!         CommonQueryParameters::builder()
//!             .format(OutputFormat::CSV)
//!             .build()?
//!     )
//!     .build()?;
//!
//! let response = client.time_series(request).await?;
//! if response.is_csv() {
//!     let data = response.parse_csv()?;
//!     // Process CSV data
//! }
//! # Ok(())
//! # }
//! ```

use crate::core::{
    PriceRequest, PriceResponse, QuoteRequest, QuoteResponse, TimeSeriesRequest, TimeSeriesResponse,
};
use fundamentals::{LogoRequest, LogoResponse};
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Display;

use errors::{Error, Result};
use http_client::HttpClient;

use derive_builder::Builder;

pub mod core;
pub mod errors;
pub mod fundamentals;
pub mod http_client;
pub mod response;

pub use response::{CsvParseable, DataResponse, ResponseFormat};

const API_URL: &str = "https://api.twelvedata.com";

/// Main client for interacting with the Twelve Data API.
///
/// The client is reusable and should typically be created once and shared across requests.
/// It uses a pluggable HTTP client implementation via the `HttpClient` trait.
///
/// # Examples
///
/// ```rust,no_run
/// use twelve_data_inav::TwelveData;
///
/// let client = TwelveData::new("your_api_key", Box::new(reqwest::Client::new()));
/// ```
pub struct TwelveData {
    api_key: String,
    client: Box<dyn HttpClient + Send + Sync>,
}

impl TwelveData {
    /// Creates a new TwelveData client.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Twelve Data API key
    /// * `client` - A boxed HTTP client implementing the `HttpClient` trait
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use twelve_data_inav::TwelveData;
    ///
    /// let client = TwelveData::new("your_api_key", Box::new(reqwest::Client::new()));
    /// ```
    pub fn new(api_key: &str, client: Box<dyn HttpClient + Send + Sync>) -> Self {
        Self {
            api_key: api_key.to_owned(),
            client,
        }
    }

    /// Fetches time series data for a given symbol.
    ///
    /// Returns historical price data (OHLCV) at the specified interval.
    /// Supports both JSON and CSV response formats.
    ///
    /// # Arguments
    ///
    /// * `req` - A `TimeSeriesRequest` configured with symbol, interval, and other parameters
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
    /// let request = TimeSeriesRequest::builder()
    ///     .symbol("AAPL".into())
    ///     .interval(Interval::Day)
    ///     .output_size(30)
    ///     .build()?;
    ///
    /// let response = client.time_series(request).await?;
    /// let data = response.parse()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn time_series(
        &self,
        req: TimeSeriesRequest,
    ) -> Result<DataResponse<TimeSeriesResponse>> {
        self.send("time_series", &req).await
    }

    /// Fetches the current quote for a given symbol.
    ///
    /// Returns real-time or delayed quote data including price, volume, and other metrics.
    ///
    /// # Arguments
    ///
    /// * `req` - A `QuoteRequest` configured with symbol and interval
    pub async fn quote(&self, req: QuoteRequest) -> Result<DataResponse<QuoteResponse>> {
        self.send("quote", &req).await
    }

    /// Fetches the current price for a given symbol.
    ///
    /// Returns a simplified response with just the current price.
    ///
    /// # Arguments
    ///
    /// * `req` - A `PriceRequest` configured with the symbol
    pub async fn price(&self, req: PriceRequest) -> Result<DataResponse<PriceResponse>> {
        self.send("price", &req).await
    }

    pub async fn logo(&self, req: LogoRequest) -> Result<DataResponse<LogoResponse>> {
        self.send("logo", &req).await
    }

    async fn send<T, U>(&self, endpoint: &str, req: &T) -> Result<DataResponse<U>>
    where
        T: serde::ser::Serialize,
        U: serde::de::DeserializeOwned + crate::response::CsvParseable,
    {
        let params = serde_urlencoded::to_string(req)?;
        let url = format!("{}/{}?{}", API_URL, endpoint, params);

        // Extract delimiter from request if present
        let delimiter = serde_json::to_value(req)
            .ok()
            .and_then(|v| v.get("delimiter").cloned())
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        let res = self.client.get(&url, &self.api_key).await?;

        if res.status != 200 {
            return Err(Error::DataError(format!("HTTP status {}", res.status)));
        }

        // Determine format from content
        let format =
            if res.body.trim_start().starts_with('{') || res.body.trim_start().starts_with('[') {
                ResponseFormat::Json
            } else {
                ResponseFormat::Csv
            };

        // For JSON responses, check for error status
        if matches!(format, ResponseFormat::Json) {
            let val: serde_json::Value = serde_json::from_str(&res.body)?;
            if let Some(status) = val.get("status") {
                if !status.is_string() {
                    return Err(Error::DataError(
                        "status value in the response is not a string".into(),
                    ));
                }
                if let Some(status_str) = status.as_str() {
                    if status_str == "error" {
                        let reason = val
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("<unknown reason>");
                        return Err(Error::DataError(reason.into()));
                    }
                }
            }
        }

        Ok(DataResponse::new(res.body, res.status, format).with_delimiter(delimiter))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Interval {
    #[serde(rename = "1min")]
    Minute,

    #[serde(rename = "5min")]
    FiveMinutes,

    #[serde(rename = "15min")]
    FifteenMinutes,

    #[serde(rename = "30min")]
    ThirtyMinutes,

    #[serde(rename = "45min")]
    FortyFiveMinutes,

    #[serde(rename = "1h")]
    Hour,

    #[serde(rename = "2h")]
    TwoHours,

    #[serde(rename = "4h")]
    FourHours,

    #[serde(rename = "1day")]
    Day,

    #[serde(rename = "1week")]
    Week,

    #[serde(rename = "1month")]
    Month,
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Interval::Minute => "1min",
                Interval::FiveMinutes => "5min",
                Interval::FifteenMinutes => "15min",
                Interval::ThirtyMinutes => "30min",
                Interval::FortyFiveMinutes => "45min",
                Interval::Hour => "1h",
                Interval::TwoHours => "2h",
                Interval::FourHours => "4h",
                Interval::Day => "1day",
                Interval::Week => "1week",
                Interval::Month => "1month",
            }
        )
    }
}

/// Type of financial instrument.
#[derive(Debug, Serialize, Deserialize)]
pub enum InstrumentType {
    Stock,
    Index,
    ETF,
    REIT,
}

/// Output format for API responses.
///
/// The format parameter is case-insensitive (e.g., "csv", "CSV", "Csv" all work).
///
/// # Examples
///
/// ```rust
/// use twelve_data_inav::OutputFormat;
///
/// let format = OutputFormat::CSV;
/// ```
#[derive(Debug, Serialize)]
pub enum OutputFormat {
    /// JSON format (default) - includes metadata and structured data
    JSON,
    /// CSV format - semicolon-delimited by default, no metadata
    CSV,
}

impl<'de> serde::Deserialize<'de> for OutputFormat {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_uppercase().as_str() {
            "JSON" => Ok(OutputFormat::JSON),
            "CSV" => Ok(OutputFormat::CSV),
            _ => Err(serde::de::Error::unknown_variant(&s, &["JSON", "CSV"])),
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::JSON
    }
}

#[derive(Debug, Serialize, Deserialize, Builder, Default)]
#[builder(pattern = "owned")]
#[skip_serializing_none]
pub struct CommonQueryParameters {
    #[builder(default, setter(strip_option))]
    pub exchange: Option<String>,

    #[builder(default, setter(strip_option))]
    pub mic_code: Option<String>,

    #[builder(default, setter(strip_option))]
    pub country: Option<String>,

    #[serde(rename = "type")]
    #[builder(default, setter(strip_option))]
    pub instrument_type: Option<InstrumentType>,

    #[serde(default)]
    #[builder(default, setter(strip_option))]
    pub format: Option<OutputFormat>,

    #[builder(default, setter(strip_option))]
    pub delimiter: Option<String>,

    #[serde(rename = "dp")]
    #[builder(default, setter(strip_option))]
    pub decimal_places: Option<u8>,

    #[builder(default, setter(strip_option))]
    pub timezone: Option<String>,
}

impl CommonQueryParameters {
    pub fn builder() -> CommonQueryParametersBuilder {
        CommonQueryParametersBuilder::default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Order {
    ASC,
    DESC,
}

#[cfg(test)]
mod test {
    use std::env;

    use tokio_test::assert_ok;

    use crate::core::TimeSeriesRequest;

    use super::*;

    #[cfg(feature = "reqwest-client")]
    fn get_client() -> Box<impl HttpClient> {
        Box::new(reqwest::Client::new())
    }

    #[cfg(feature = "surf-client")]
    fn get_client() -> Box<impl HttpClient> {
        Box::new(surf::Client::new())
    }

    fn get_api_key() -> String {
        env::var("TWELVE_DATA_API_KEY").unwrap()
    }

    #[test]
    pub fn time_series() {
        let td = TwelveData::new(&get_api_key(), get_client());

        let res = tokio_test::block_on(
            td.time_series(
                TimeSeriesRequest::builder()
                    .symbol("TSLA".into())
                    .interval(Interval::Day)
                    .output_size(10)
                    .build()
                    .unwrap(),
            ),
        );

        assert_ok!(&res);

        let response = res.unwrap();
        assert!(response.is_json());

        let parsed = response.parse().unwrap();
        assert_eq!(10, parsed.values.len());
    }

    #[test]
    pub fn time_series_csv() {
        use crate::core::TimeSeriesRequest;

        let td = TwelveData::new(&get_api_key(), get_client());

        let req = TimeSeriesRequest {
            common: CommonQueryParameters {
                format: Some(OutputFormat::CSV),
                ..Default::default()
            },
            symbol: Some("AAPL".into()),
            isin: None,
            figi: None,
            cusip: None,
            interval: Interval::Day,
            output_size: Some(5),
            order: None,
            start_date: None,
            end_date: None,
            previous_close: None,
        };

        let res = tokio_test::block_on(td.time_series(req));

        assert_ok!(&res);

        let response = res.unwrap();
        assert!(response.is_csv(), "Response should be CSV format");

        // Test raw CSV access
        let raw_csv = response.raw();
        println!("CSV Response:\n{}", raw_csv);
        assert!(!raw_csv.is_empty(), "CSV should not be empty");

        // Test automatic parsing
        let parsed = response.parse().unwrap();
        assert!(parsed.meta.is_none(), "CSV response should not have meta");
        assert_eq!("ok", parsed.status);
        assert_eq!(5, parsed.values.len(), "Should have 5 values");

        // Verify first value has proper f64 types
        let first = &parsed.values[0];
        assert!(first.open > 0.0);
        assert!(first.high > 0.0);
        assert!(first.low > 0.0);
        assert!(first.close > 0.0);
    }

    #[test]
    pub fn quote_csv() {
        use crate::core::QuoteRequest;

        let td = TwelveData::new(&get_api_key(), get_client());

        let req = QuoteRequest {
            common: CommonQueryParameters {
                format: Some(OutputFormat::CSV),
                ..Default::default()
            },
            symbol: Some("AAPL".into()),
            isin: None,
            figi: None,
            cusip: None,
            interval: Interval::Day,
            volume_time_period: None,
            end_of_day: None,
            rolling_period: None,
        };

        let res = tokio_test::block_on(td.quote(req));

        assert_ok!(&res);

        let response = res.unwrap();
        assert!(response.is_csv(), "Response should be CSV format");

        // Test parsing
        let parsed = response.parse_csv().unwrap();
        assert!(!parsed.symbol.is_empty());
        assert!(parsed.close > 0.0, "Close price should be positive");
    }

    #[test]
    pub fn price_csv() {
        use crate::core::PriceRequest;

        let td = TwelveData::new(&get_api_key(), get_client());

        let req = PriceRequest {
            common: CommonQueryParameters {
                format: Some(OutputFormat::CSV),
                ..Default::default()
            },
            symbol: Some("AAPL".into()),
            isin: None,
            figi: None,
            cusip: None,
            output_size: None,
            order: None,
            start_date: None,
            end_date: None,
            previous_close: None,
        };

        let res = tokio_test::block_on(td.price(req));

        assert_ok!(&res);

        let response = res.unwrap();
        assert!(response.is_csv(), "Response should be CSV format");

        // Test parsing
        let parsed = response.parse_csv().unwrap();
        assert!(parsed.price > 0.0, "Price should be positive");
    }

    #[test]
    pub fn parse_method_auto_detects_format() {
        let td = TwelveData::new(&get_api_key(), get_client());

        // Test with JSON
        let json_res = tokio_test::block_on(
            td.time_series(
                TimeSeriesRequest::builder()
                    .symbol("AAPL".into())
                    .interval(Interval::Day)
                    .output_size(3)
                    .build()
                    .unwrap(),
            ),
        )
        .unwrap();

        let json_parsed = json_res.parse().unwrap();
        assert!(json_parsed.meta.is_some());
        assert_eq!(3, json_parsed.values.len());

        // Test with CSV
        let csv_req = TimeSeriesRequest {
            common: CommonQueryParameters {
                format: Some(OutputFormat::CSV),
                ..Default::default()
            },
            symbol: Some("AAPL".into()),
            isin: None,
            figi: None,
            cusip: None,
            interval: Interval::Day,
            output_size: Some(3),
            order: None,
            start_date: None,
            end_date: None,
            previous_close: None,
        };

        let csv_res = tokio_test::block_on(td.time_series(csv_req)).unwrap();

        let csv_parsed = csv_res.parse().unwrap();
        assert!(csv_parsed.meta.is_none());
        assert_eq!(3, csv_parsed.values.len());
    }

    #[test]
    pub fn output_format_case_insensitive() {
        // Test that OutputFormat deserialization is case-insensitive
        #[derive(Deserialize)]
        struct TestParams {
            format: OutputFormat,
        }

        // Test lowercase
        let result: TestParams = serde_urlencoded::from_str("format=csv").unwrap();
        assert!(matches!(result.format, OutputFormat::CSV));

        // Test uppercase
        let result: TestParams = serde_urlencoded::from_str("format=CSV").unwrap();
        assert!(matches!(result.format, OutputFormat::CSV));

        // Test mixed case
        let result: TestParams = serde_urlencoded::from_str("format=Csv").unwrap();
        assert!(matches!(result.format, OutputFormat::CSV));

        // Test JSON
        let result: TestParams = serde_urlencoded::from_str("format=json").unwrap();
        assert!(matches!(result.format, OutputFormat::JSON));

        let result: TestParams = serde_urlencoded::from_str("format=JSON").unwrap();
        assert!(matches!(result.format, OutputFormat::JSON));
    }
}
