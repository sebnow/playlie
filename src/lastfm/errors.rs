use hyper;
use serde::de::{self, Visitor};
use serde_json;
use std::convert::From;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// An error occurred while parsing the response
    ParsingError(serde_json::error::Error),
    /// An error occurred during the request
    HTTPError(hyper::error::Error),
    /// An error occurred from the API
    APIError(ErrorResponse),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::ParsingError(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Error::HTTPError(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorCode {
    /// Invalid service -This service does not exist
    InvalidService = 2,
    /// Invalid Method - No method with that name in this package
    InvalidMethod = 3,
    /// Authentication Failed - You do not have permissions to access the service
    AuthenticationFailed = 4,
    /// Invalid format - This service doesn't exist in that format
    InvalidFormat = 5,
    /// Invalid parameters - Your request is missing a required parameter
    InvalidParameters = 6,
    /// Invalid resource specified
    InvalidResource = 7,
    /// Operation failed - Most likely the backend service failed. Please try again.
    OperationFailed = 8,
    /// Invalid session key - Please re-authenticate
    InvalidSessionKey = 9,
    /// Invalid API key - You must be granted a valid key by last.fm
    InvalidAPIKey = 10,
    /// Service Offline - This service is temporarily offline. Try again later.
    ServiceOffline = 11,
    /// Subscribers Only - This station is only available to paid last.fm subscribers
    SubscribersOnly = 12,
    /// Invalid method signature supplied
    InvalidMethodSignature = 13,
    /// Unauthorized Token - This token has not been authorized
    UnauthorizedToken = 14,
    /// This item is not available for streaming.
    StreamingNotAvailable = 15,
    /// The service is temporarily unavailable, please try again.
    ServiceTemporarilyUnavailable = 16,
    /// Login: User requires to be logged in
    RequiresLogin = 17,
    /// Trial Expired - This user has no free radio plays left. Subscription required.
    TrialExpired = 18,
    /// Not Enough Content - There is not enough content to play this station
    NotEnoughContent = 20,
    /// Not Enough Members - This group does not have enough members for radio
    NotEnoughMembers = 21,
    /// Not Enough Fans - This artist does not have enough fans for for radio
    NotEnoughFans = 22,
    /// Not Enough Neighbours - There are not enough neighbours for radio
    NotEnoughNeighbours = 23,
    /// No Peak Radio - This user is not allowed to listen to radio during peak usage
    NoPeakRadio = 24,
    /// Radio Not Found - Radio station not found
    RadioNotFound = 25,
    /// API Key Suspended - This application is not allowed to make requests to the web services
    APIKeySuspended = 26,
    /// Deprecated - This type of request is no longer supported
    Deprecated = 27,
    /// Rate Limit Exceded - Your IP has made too many requests in a short period, exceeding our API guidelines
    RateLimitExceeded = 29,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct InvalidErrorCode(u64);

impl TryFrom<u64> for ErrorCode {
    type Error = InvalidErrorCode;

    fn try_from(u: u64) -> Result<Self, Self::Error> {
        match u {
            2 => Ok(ErrorCode::InvalidService),
            3 => Ok(ErrorCode::InvalidMethod),
            4 => Ok(ErrorCode::AuthenticationFailed),
            5 => Ok(ErrorCode::InvalidFormat),
            6 => Ok(ErrorCode::InvalidParameters),
            7 => Ok(ErrorCode::InvalidResource),
            8 => Ok(ErrorCode::OperationFailed),
            9 => Ok(ErrorCode::InvalidSessionKey),
            10 => Ok(ErrorCode::InvalidAPIKey),
            11 => Ok(ErrorCode::ServiceOffline),
            12 => Ok(ErrorCode::SubscribersOnly),
            13 => Ok(ErrorCode::InvalidMethodSignature),
            14 => Ok(ErrorCode::UnauthorizedToken),
            15 => Ok(ErrorCode::StreamingNotAvailable),
            16 => Ok(ErrorCode::ServiceTemporarilyUnavailable),
            17 => Ok(ErrorCode::RequiresLogin),
            18 => Ok(ErrorCode::TrialExpired),
            20 => Ok(ErrorCode::NotEnoughContent),
            21 => Ok(ErrorCode::NotEnoughMembers),
            22 => Ok(ErrorCode::NotEnoughFans),
            23 => Ok(ErrorCode::NotEnoughNeighbours),
            24 => Ok(ErrorCode::NoPeakRadio),
            25 => Ok(ErrorCode::RadioNotFound),
            26 => Ok(ErrorCode::APIKeySuspended),
            27 => Ok(ErrorCode::Deprecated),
            29 => Ok(ErrorCode::RateLimitExceeded),
            _ => Err(InvalidErrorCode(u as u64)),
        }
    }
}

// Manually implement ErrorCode deserialization from an interger, as serde does not
// currently support deserializing to a C-style enum; https://github.com/serde-rs/json/issues/349
impl<'de> de::Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<ErrorCode, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_i32(ErrorCodeVisitor)
    }
}

struct ErrorCodeVisitor;

impl<'de> Visitor<'de> for ErrorCodeVisitor {
    type Value = ErrorCode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 2 and 29")
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E> where E: de::Error {
        ErrorCode::try_from(value as u64).map_err(|e| E::custom(format!("invalid error code: {}", e.0)))
    }

    fn visit_u32<E>(self, value: u32) -> Result<ErrorCode, E> where E: de::Error {
        ErrorCode::try_from(value as u64).map_err(|e| E::custom(format!("invalid error code: {}", e.0)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<ErrorCode, E> where E: de::Error {
        ErrorCode::try_from(value as u64).map_err(|e| E::custom(format!("invalid error code: {}", e.0)))
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ErrorResponse {
    error: ErrorCode,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn deserialize_api_error() {
        let json = json!({
            "error": 10,
            "message": "Invalid API Key"
        });

        let err: ErrorResponse = serde_json::from_value(json).unwrap();
        assert_eq!(
            err,
            ErrorResponse {
                error: ErrorCode::InvalidAPIKey,
                message: "Invalid API Key".into(),
            },
        );
    }

    #[test]
    fn error_code_try_from() {
        assert_eq!(Err(InvalidErrorCode(255)), ErrorCode::try_from(255));
        assert_eq!(Err(InvalidErrorCode(4294967295)), ErrorCode::try_from(4294967295));
        assert_eq!(Ok(ErrorCode::InvalidFormat), ErrorCode::try_from(5));
    }
}
