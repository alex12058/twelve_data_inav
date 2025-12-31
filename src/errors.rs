use std::{error::Error as StdError, fmt::Display, result::Result as StdResult};

use serde::de::DeserializeOwned;

use crate::http_client::Response;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "reqwest-client")]
    ReqwestError(reqwest::Error),

    #[cfg(feature = "surf-client")]
    SurfError(surf::Error),

    #[cfg(feature = "wreq-client")]
    WreqError(wreq::Error),

    QueryConstruction(serde_urlencoded::ser::Error),

    ResponseParsing(serde_json::Error),

    DataError(String),
}

pub type Result<T> = StdResult<T, Error>;

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            #[cfg(feature = "reqwest-client")]
            Error::ReqwestError(e) => Some(e),
            #[cfg(feature = "surf-client")]
            Error::SurfError(e) => Some(e),
            #[cfg(feature = "wreq-client")]
            Error::WreqError(e) => Some(e),
            Error::QueryConstruction(e) => Some(e),
            Error::ResponseParsing(e) => Some(e),
            Error::DataError(_) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "reqwest-client")]
            Error::ReqwestError(e) => write!(
                f,
                "HTTP error (status {})",
                e.status()
                    .map_or_else(|| "unknown".to_owned(), |s| s.as_str().into())
            ),
            #[cfg(feature = "surf-client")]
            Error::SurfError(e) => write!(
                f,
                "HTTP error (status {})",
                e.status()
                    .map_or_else(|| "unknown".to_owned(), |s| s.as_str().into())
            ),
            #[cfg(feature = "wreq-client")]
            Error::WreqError(e) => write!(
                f,
                "HTTP error (status {})",
                e.status()
                    .map_or_else(|| "unknown".to_owned(), |s| s.as_str().into())
            ),
            Error::QueryConstruction(_) => write!(f, "query construction error"),
            Error::ResponseParsing(_) => write!(f, "failed to parse the output"),
            Error::DataError(reason) => write!(f, "failed to obtain data: {}", reason),
        }
    }
}

#[cfg(feature = "reqwest-client")]
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

#[cfg(feature = "surf-client")]
impl From<surf::Error> for Error {
    fn from(e: surf::Error) -> Self {
        Self::SurfError(e)
    }
}

#[cfg(feature = "wreq-client")]
impl From<wreq::Error> for Error {
    fn from(e: wreq::Error) -> Self {
        Self::WreqError(e)
    }
}

impl From<serde_urlencoded::ser::Error> for Error {
    fn from(e: serde_urlencoded::ser::Error) -> Self {
        Self::QueryConstruction(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::ResponseParsing(e)
    }
}

impl<T: DeserializeOwned> From<Response> for Result<T> {
    fn from(res: Response) -> Self {
        if res.status == 200 {
            Ok(serde_json::from_str::<T>(&res.body)?)
        } else {
            Err(Error::DataError(format!("status {}", res.status)))
        }
    }
}
