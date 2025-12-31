use std::ops::Range;

use chrono::{NaiveDate, NaiveDateTime};
use derive_builder::Builder;
use serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

use super::{CommonQueryParameters, Interval, Order};

#[derive(Debug, Serialize, Deserialize, Builder)]
#[builder(pattern = "owned", build_fn(validate = "Self::validate"))]
#[skip_serializing_none]
pub struct TimeSeriesRequest {
    #[serde(flatten)]
    #[builder(default)]
    pub common: CommonQueryParameters,

    #[builder(default, setter(strip_option))]
    pub symbol: Option<String>,

    #[builder(default, setter(strip_option))]
    pub isin: Option<String>,

    #[builder(default, setter(strip_option))]
    pub figi: Option<String>,

    #[builder(default, setter(strip_option))]
    pub cusip: Option<String>,

    pub interval: Interval,

    #[serde(rename = "outputsize")]
    #[builder(default, setter(strip_option))]
    pub output_size: Option<u16>,

    #[builder(default, setter(strip_option))]
    pub order: Option<Order>,

    #[builder(default, setter(strip_option))]
    pub start_date: Option<NaiveDateTime>,

    #[builder(default, setter(strip_option))]
    pub end_date: Option<NaiveDateTime>,

    #[builder(default, setter(strip_option))]
    pub previous_close: Option<bool>,
}

impl TimeSeriesRequest {
    pub fn builder() -> TimeSeriesRequestBuilder {
        TimeSeriesRequestBuilder::default()
    }
}

impl TimeSeriesRequestBuilder {
    fn validate(&self) -> Result<(), String> {
        if self.symbol.is_none()
            && self.isin.is_none()
            && self.figi.is_none()
            && self.cusip.is_none()
        {
            Err(
                "At least one identifier (symbol, isin, figi, or cusip) must be provided"
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }
}

/// Response containing time series data (OHLCV historical prices).
///
/// The structure varies depending on the response format:
/// - **JSON format**: Includes `meta` field with symbol information and metadata
/// - **CSV format**: `meta` field is `None`, only `values` are populated
///
/// # Fields
///
/// * `meta` - Metadata about the symbol (only available in JSON format)
/// * `status` - Response status ("ok" or "error")
/// * `values` - Vector of time series quotes with OHLCV data
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
///     .build()?;
///
/// let response = client.time_series(request).await?;
/// let data = response.parse()?;
///
/// // meta is Some in JSON format
/// if let Some(meta) = data.meta {
///     println!("Symbol: {}", meta.symbol);
/// }
///
/// for quote in data.values {
///     println!("Date: {}, Close: {}", quote.datetime, quote.close);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<TimeSeriesMeta>,
    pub status: String,
    pub values: Vec<TimeSeriesQuote>,
}

impl crate::response::CsvParseable for TimeSeriesResponse {
    type CsvRecord = TimeSeriesQuote;

    fn from_csv_records(records: Vec<Self::CsvRecord>) -> Self {
        Self {
            meta: None,
            status: "ok".to_string(),
            values: records,
        }
    }
}

/// Metadata for time series responses.
///
/// **Note**: This is only available when using JSON format (`format=JSON`).
/// CSV responses do not include metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesMeta {
    pub symbol: String,
    pub interval: Interval,
    pub currency: String,
    pub exchange_timezone: String,
    pub exchange: String,
    pub mic_code: String,

    #[serde(rename = "type")]
    pub instrument_type: String,
}

/// A single time series data point with OHLCV (Open, High, Low, Close, Volume) data.
///
/// This structure is available in both JSON and CSV formats.
///
/// # Fields
///
/// * `datetime` - Timestamp of the data point
/// * `open` - Opening price
/// * `high` - Highest price during the period
/// * `low` - Lowest price during the period
/// * `close` - Closing price
/// * `volume` - Trading volume
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesQuote {
    #[serde(deserialize_with = "deserialize_td_datetime")]
    pub datetime: NaiveDateTime,

    #[serde_as(as = "DisplayFromStr")]
    pub open: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub low: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub close: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
#[builder(pattern = "owned", build_fn(validate = "Self::validate"))]
#[skip_serializing_none]
pub struct QuoteRequest {
    #[serde(flatten)]
    #[builder(default)]
    pub common: CommonQueryParameters,

    #[builder(default, setter(strip_option))]
    pub symbol: Option<String>,

    #[builder(default, setter(strip_option))]
    pub isin: Option<String>,

    #[builder(default, setter(strip_option))]
    pub figi: Option<String>,

    #[builder(default, setter(strip_option))]
    pub cusip: Option<String>,

    pub interval: Interval,

    #[builder(default, setter(strip_option))]
    pub volume_time_period: Option<u32>,

    #[serde(rename = "eod")]
    #[builder(default, setter(strip_option))]
    pub end_of_day: Option<bool>,

    #[builder(default, setter(strip_option))]
    pub rolling_period: Option<u8>,
}

impl QuoteRequest {
    pub fn builder() -> QuoteRequestBuilder {
        QuoteRequestBuilder::default()
    }
}

impl QuoteRequestBuilder {
    fn validate(&self) -> Result<(), String> {
        if self.symbol.is_none()
            && self.isin.is_none()
            && self.figi.is_none()
            && self.cusip.is_none()
        {
            Err(
                "At least one identifier (symbol, isin, figi, or cusip) must be provided"
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }
}

/// Real-time or delayed quote data for a symbol.
///
/// **CSV Format Limitations**: When using `format=CSV`, some fields may not be populated
/// or may contain default values. The following fields are reliably available in CSV:
/// - `symbol`, `datetime`, `open`, `high`, `low`, `close`, `volume`
///
/// Fields like `name`, `exchange`, `fifty_two_week` stats may be empty or have default values in CSV format.
///
/// # Examples
///
/// ```rust,no_run
/// # use twelve_data_inav::{TwelveData, core::QuoteRequest, Interval};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
/// let request = QuoteRequest::builder()
///     .symbol("AAPL".into())
///     .interval(Interval::Day)
///     .build()?;
///
/// let response = client.quote(request).await?;
/// let quote = response.parse()?;
///
/// println!("Symbol: {}", quote.symbol);
/// println!("Current Price: {}", quote.close);
/// println!("Change: {} ({}%)", quote.change, quote.percent_change);
/// # Ok(())
/// # }
/// ```
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub mic_code: String,
    pub currency: String,
    pub timestamp: i64,

    #[serde(deserialize_with = "deserialize_td_datetime")]
    pub datetime: NaiveDateTime,

    #[serde_as(as = "DisplayFromStr")]
    pub open: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub low: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub close: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub volume: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub previous_close: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub change: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub percent_change: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub average_volume: f64,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub rolling_1d_change: Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub rolling_7d_change: Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub rolling_period_change: Option<f64>,
    #[serde(default)]
    pub is_market_open: bool,
    #[serde(default)]
    pub fifty_two_week: FiftyTwoWeekStats,
}

impl crate::response::CsvParseable for QuoteResponse {
    type CsvRecord = Self;

    fn from_csv_records(records: Vec<Self::CsvRecord>) -> Self {
        records.into_iter().next().unwrap_or_default()
    }
}

impl Default for QuoteResponse {
    fn default() -> Self {
        use chrono::NaiveDate;
        Self {
            symbol: String::new(),
            name: String::new(),
            exchange: String::new(),
            mic_code: String::new(),
            currency: String::new(),
            timestamp: 0,
            datetime: NaiveDate::from_ymd_opt(1970, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            previous_close: 0.0,
            change: 0.0,
            percent_change: 0.0,
            average_volume: 0.0,
            rolling_1d_change: None,
            rolling_7d_change: None,
            rolling_period_change: None,
            is_market_open: false,
            fifty_two_week: FiftyTwoWeekStats::default(),
        }
    }
}

/// 52-week statistics for a quote.
///
/// **Note**: This field may contain default/zero values when using CSV format.
/// It is most reliable in JSON format responses.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FiftyTwoWeekStats {
    #[serde_as(as = "DisplayFromStr")]
    pub low: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub low_change: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high_change: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub low_change_percent: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high_change_percent: f64,
    #[serde(deserialize_with = "deserialize_td_range")]
    range: Range<f64>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
#[builder(pattern = "owned", build_fn(validate = "Self::validate"))]
#[skip_serializing_none]
pub struct PriceRequest {
    #[serde(flatten)]
    #[builder(default)]
    pub common: CommonQueryParameters,

    #[builder(default, setter(strip_option))]
    pub symbol: Option<String>,

    #[builder(default, setter(strip_option))]
    pub isin: Option<String>,

    #[builder(default, setter(strip_option))]
    pub figi: Option<String>,

    #[builder(default, setter(strip_option))]
    pub cusip: Option<String>,

    #[serde(rename = "outputsize")]
    #[builder(default, setter(strip_option))]
    pub output_size: Option<u16>,

    #[builder(default, setter(strip_option))]
    pub order: Option<Order>,

    #[builder(default, setter(strip_option))]
    pub start_date: Option<NaiveDateTime>,

    #[builder(default, setter(strip_option))]
    pub end_date: Option<NaiveDateTime>,

    #[builder(default, setter(strip_option))]
    pub previous_close: Option<bool>,
}

impl PriceRequest {
    pub fn builder() -> PriceRequestBuilder {
        PriceRequestBuilder::default()
    }
}

impl PriceRequestBuilder {
    fn validate(&self) -> Result<(), String> {
        if self.symbol.is_none()
            && self.isin.is_none()
            && self.figi.is_none()
            && self.cusip.is_none()
        {
            Err(
                "At least one identifier (symbol, isin, figi, or cusip) must be provided"
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }
}

/// Simple price response containing just the current price.
///
/// This response type works identically in both JSON and CSV formats.
///
/// # Examples
///
/// ```rust,no_run
/// # use twelve_data_inav::{TwelveData, core::PriceRequest};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = TwelveData::new("key", Box::new(reqwest::Client::new()));
/// let request = PriceRequest::builder()
///     .symbol("AAPL".into())
///     .build()?;
///
/// let response = client.price(request).await?;
/// let price_data = response.parse()?;
///
/// println!("Current price: {}", price_data.price);
/// # Ok(())
/// # }
/// ```
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PriceResponse {
    #[serde_as(as = "DisplayFromStr")]
    pub price: f64,
}

impl crate::response::CsvParseable for PriceResponse {
    type CsvRecord = Self;

    fn from_csv_records(records: Vec<Self::CsvRecord>) -> Self {
        records.into_iter().next().unwrap_or_default()
    }
}

pub fn deserialize_td_datetime<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    d.deserialize_str(TdDateTimeVisitor)
}

struct TdDateTimeVisitor;
impl<'de> serde::de::Visitor<'de> for TdDateTimeVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a datetime string in the TwelveData format (%Y-%m-%d %H:%M:%S or %Y-%m-%d)"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S") {
            Ok(ndt) => Ok(ndt),
            Err(_) => match NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                Ok(nd) => nd
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| E::custom("Failed to create datetime with midnight time")),
                Err(e) => Err(E::custom(format!(
                    "unexpected date time format of {}: {}",
                    v, e
                ))),
            },
        }
    }
}

pub fn deserialize_td_range<'de, D>(d: D) -> Result<Range<f64>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    d.deserialize_str(TdRangeVisitor)
}

struct TdRangeVisitor;
impl<'de> serde::de::Visitor<'de> for TdRangeVisitor {
    type Value = Range<f64>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expected TwelveData range as string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let idx = v
            .find(" - ")
            .ok_or_else(|| E::custom("couldn't find the range separator"))?;
        let (first, second) = v.split_at(idx);
        let second = &second[3..];

        let first = first.parse::<f64>().map_err(|e| {
            E::custom(format!(
                "failed to parse the first range value: {}",
                e.to_string()
            ))
        })?;
        let second = second.parse::<f64>().map_err(|e| {
            E::custom(format!(
                "failed to parse the second range value: {}",
                e.to_string()
            ))
        })?;
        Ok(Range {
            start: first,
            end: second,
        })
    }
}

#[cfg(test)]
mod test {
    use tokio_test::assert_ok;

    use super::*;

    #[test]
    pub fn test_timeseries_response() {
        let response = r#"{"meta":{"currency":"USD","exchange":"NASDAQ","exchange_timezone":"America/New_York","interval":"1day","mic_code":"XNGS","symbol":"TSLA","type":"Common Stock"},"status":"ok","values":[{"close":"308.73001","datetime":"2022-09-20","high":"313.32999","low":"305.57999","open":"306.91501","volume":"231261"},{"close":"309.07001","datetime":"2022-09-19","high":"309.84000","low":"297.79999","open":"300.09000","volume":"60060200"},{"close":"303.35001","datetime":"2022-09-16","high":"303.70999","low":"295.60001","open":"299.60999","volume":"86949500"},{"close":"303.75000","datetime":"2022-09-15","high":"309.12000","low":"300.72000","open":"301.82999","volume":"64795500"},{"close":"302.60999","datetime":"2022-09-14","high":"306.00000","low":"291.64001","open":"292.23999","volume":"72628700"},{"close":"292.13000","datetime":"2022-09-13","high":"297.39999","low":"290.39999","open":"292.89999","volume":"68229600"},{"close":"304.42001","datetime":"2022-09-12","high":"305.48999","low":"300.39999","open":"300.72000","volume":"48674600"},{"close":"299.67999","datetime":"2022-09-09","high":"299.85001","low":"291.25000","open":"291.67001","volume":"54338100"},{"close":"289.26001","datetime":"2022-09-08","high":"289.50000","low":"279.76001","open":"281.29999","volume":"53713100"},{"close":"283.70001","datetime":"2022-09-07","high":"283.84000","low":"272.26999","open":"273.10001","volume":"50028900"}]}"#;

        let response = serde_json::from_str::<TimeSeriesResponse>(response);

        assert_ok!(&response);

        let res = &response.unwrap();
        assert!(res.meta.is_some());
        assert_eq!(10, res.values.len());
    }

    #[test]
    pub fn test_quote_response() {
        let response = r#"{"symbol":"AAPL","name":"Apple Inc","exchange":"NASDAQ","mic_code":"XNGS","currency":"USD","datetime":"2022-09-20","timestamp":1663703999,"open":"153.39999","high":"158.08000","low":"153.08000","close":"156.89999","volume":"107547900","previous_close":"154.48000","change":"2.42000","percent_change":"1.56654","average_volume":"99764040","is_market_open":false,"fifty_two_week":{"low":"129.03999","high":"182.94000","low_change":"27.86000","high_change":"-26.04001","low_change_percent":"21.59021","high_change_percent":"-14.23418","range":"129.039993 - 182.940002"}}"#;

        let response = serde_json::from_str::<QuoteResponse>(response);

        assert_ok!(&response);

        let res = response.unwrap();
        let range = res.fifty_two_week.range;
        assert_eq!(range.start, 129.039993);
        assert_eq!(range.end, 182.940002);
    }
}
