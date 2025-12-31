//! Response handling for both JSON and CSV formats.
//!
//! This module provides the `DataResponse<T>` type which wraps API responses
//! and provides methods for parsing both JSON and CSV formats into strongly-typed structures.

use crate::errors::{Error, Result};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

/// Format of the API response.
#[derive(Debug, Clone)]
pub enum ResponseFormat {
    /// JSON response format
    Json,
    /// CSV response format
    Csv,
}

impl ResponseFormat {
    /// Returns the HTTP Content-Type header value for this format.
    pub fn content_type(&self) -> &'static str {
        match self {
            ResponseFormat::Json => "application/json",
            ResponseFormat::Csv => "text/csv",
        }
    }
}

/// Trait for types that can be parsed from CSV records.
///
/// Implement this trait to enable CSV parsing for custom response types.
pub trait CsvParseable {
    /// The intermediate CSV record type used during deserialization.
    type CsvRecord: DeserializeOwned;
    /// Converts a vector of CSV records into the final response type.
    fn from_csv_records(records: Vec<Self::CsvRecord>) -> Self;
}

/// A generic wrapper for API responses supporting both JSON and CSV formats.
///
/// This type provides a unified interface for handling different response formats
/// and includes methods for format detection, parsing, and raw access.
///
/// # Type Parameters
///
/// * `T` - The response type, must implement both `DeserializeOwned` and `CsvParseable`
///
/// # Examples
///
/// ```rust,no_run
/// # use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
/// # let request = TimeSeriesRequest::builder().symbol("AAPL".into()).interval(Interval::Day).build()?;
/// let response = client.time_series(request).await?;
///
/// // Check format
/// if response.is_csv() {
///     let data = response.parse_csv()?;
/// } else {
///     let data = response.parse_json()?;
/// }
///
/// // Or use auto-detection
/// let data = response.parse()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct DataResponse<T> {
    pub raw_body: String,
    pub status_code: u16,
    pub format: ResponseFormat,
    pub delimiter: Option<String>,
    _phantom: PhantomData<T>,
}

impl<T> DataResponse<T>
where
    T: DeserializeOwned + CsvParseable,
{
    /// Creates a new DataResponse.
    ///
    /// This is typically called internally by the client, not by end users.
    pub fn new(raw_body: String, status_code: u16, format: ResponseFormat) -> Self {
        Self {
            raw_body,
            status_code,
            format,
            delimiter: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the CSV delimiter for parsing.
    ///
    /// If not set, defaults to semicolon (`;`) which is the Twelve Data API default.
    pub fn with_delimiter(mut self, delimiter: Option<String>) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Returns the format of this response.
    pub fn format(&self) -> &ResponseFormat {
        &self.format
    }

    /// Returns `true` if this response is in JSON format.
    pub fn is_json(&self) -> bool {
        matches!(self.format, ResponseFormat::Json)
    }

    /// Returns `true` if this response is in CSV format.
    pub fn is_csv(&self) -> bool {
        matches!(self.format, ResponseFormat::Csv)
    }

    /// Returns the raw response body as a string.
    ///
    /// Useful for forwarding responses or custom parsing.
    pub fn raw(&self) -> &str {
        &self.raw_body
    }

    /// Parses the response as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the response is not in JSON format or if JSON parsing fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
    /// # let request = TimeSeriesRequest::builder().symbol("AAPL".into()).interval(Interval::Day).build()?;
    /// let response = client.time_series(request).await?;
    /// let data = response.parse_json()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_json(&self) -> Result<T> {
        if !self.is_json() {
            return Err(Error::DataError(
                "Cannot parse JSON from CSV response".to_string(),
            ));
        }
        Ok(serde_json::from_str(&self.raw_body)?)
    }

    /// Parses the response as CSV.
    ///
    /// Uses the delimiter specified in the request, or defaults to semicolon (`;`).
    ///
    /// # Errors
    ///
    /// Returns an error if the response is not in CSV format or if CSV parsing fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval, OutputFormat, CommonQueryParameters};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
    /// let request = TimeSeriesRequest::builder()
    ///     .symbol("AAPL".into())
    ///     .interval(Interval::Day)
    ///     .common(CommonQueryParameters::builder().format(OutputFormat::CSV).build()?)
    ///     .build()?;
    /// let response = client.time_series(request).await?;
    /// let data = response.parse_csv()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_csv(&self) -> Result<T> {
        if !self.is_csv() {
            return Err(Error::DataError(
                "Cannot parse CSV from JSON response".to_string(),
            ));
        }

        // Use provided delimiter or default to semicolon
        let delimiter = self
            .delimiter
            .as_ref()
            .and_then(|d| d.chars().next())
            .unwrap_or(';');

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter as u8)
            .from_reader(self.raw_body.as_bytes());
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: T::CsvRecord = result.map_err(|e| Error::DataError(e.to_string()))?;
            records.push(record);
        }

        Ok(T::from_csv_records(records))
    }

    /// Automatically parses the response based on its format.
    ///
    /// This is the recommended method as it handles both JSON and CSV responses
    /// without requiring format-specific logic.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
    /// # let request = TimeSeriesRequest::builder().symbol("AAPL".into()).interval(Interval::Day).build()?;
    /// let response = client.time_series(request).await?;
    /// let data = response.parse()?; // Works for both JSON and CSV
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(&self) -> Result<T> {
        match self.format {
            ResponseFormat::Json => self.parse_json(),
            ResponseFormat::Csv => self.parse_csv(),
        }
    }

    /// Converts the response into its component parts.
    ///
    /// Returns `(status_code, content_type, body)` tuple for custom response handling.
    pub fn into_parts(self) -> (u16, &'static str, String) {
        (self.status_code, self.format.content_type(), self.raw_body)
    }
}

#[cfg(feature = "axum")]
/// Converts a `DataResponse` into an Axum HTTP response.
///
/// This implementation is only available when the `axum` feature is enabled.
/// It automatically sets the correct status code and content-type header based on the response format.
///
/// # Examples
///
/// ```rust,no_run
/// # #[cfg(feature = "axum")]
/// # {
/// use axum::response::IntoResponse;
/// # use twelve_data_inav::{TwelveData, core::TimeSeriesRequest, Interval};
/// # async fn handler() -> axum::response::Response {
/// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
/// # let request = TimeSeriesRequest::builder().symbol("AAPL".into()).interval(Interval::Day).build().unwrap();
/// let response = client.time_series(request).await.unwrap();
/// response.into_response()
/// # }
/// # }
/// ```
impl<T> axum::response::IntoResponse for DataResponse<T>
where
    T: DeserializeOwned + CsvParseable,
{
    fn into_response(self) -> axum::response::Response {
        use axum::http::{header, StatusCode};

        let (status, content_type, body) = self.into_parts();
        (
            StatusCode::from_u16(status).unwrap_or(StatusCode::OK),
            [(header::CONTENT_TYPE, content_type)],
            body,
        )
            .into_response()
    }
}
