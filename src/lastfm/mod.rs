use hyper;
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use serde::de;
use serde_json;
use std::convert::From;

static BASE_URL: &'static str = "http://ws.audioscrobbler.com/2.0";

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

#[derive(Debug)]
pub enum ErrorCode {
    /// This error does not exist
    Generic = 1,
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

#[derive(Debug)]
pub struct ErrorResponse {
    error: ErrorCode,
    message: String,
}

// Identical to ErrorResponse but using a plain integer for the error code, because serde does not
// currently support deserializing to a C-style enum; https://github.com/serde-rs/json/issues/349
#[derive(Deserialize, Debug, PartialEq)]
struct ProxyErrorResponse {
    error: u8,
    message: String,
}

impl From<ProxyErrorResponse> for ErrorResponse {
    fn from(err: ProxyErrorResponse) -> ErrorResponse {
        let code = match err.error {
            2 => ErrorCode::InvalidService,
            3 => ErrorCode::InvalidMethod,
            4 => ErrorCode::AuthenticationFailed,
            5 => ErrorCode::InvalidFormat,
            6 => ErrorCode::InvalidParameters,
            7 => ErrorCode::InvalidResource,
            8 => ErrorCode::OperationFailed,
            9 => ErrorCode::InvalidSessionKey,
            10 => ErrorCode::InvalidAPIKey,
            11 => ErrorCode::ServiceOffline,
            12 => ErrorCode::SubscribersOnly,
            13 => ErrorCode::InvalidMethodSignature,
            14 => ErrorCode::UnauthorizedToken,
            15 => ErrorCode::StreamingNotAvailable,
            16 => ErrorCode::ServiceTemporarilyUnavailable,
            17 => ErrorCode::RequiresLogin,
            18 => ErrorCode::TrialExpired,
            20 => ErrorCode::NotEnoughContent,
            21 => ErrorCode::NotEnoughMembers,
            22 => ErrorCode::NotEnoughFans,
            23 => ErrorCode::NotEnoughNeighbours,
            24 => ErrorCode::NoPeakRadio,
            25 => ErrorCode::RadioNotFound,
            26 => ErrorCode::APIKeySuspended,
            27 => ErrorCode::Deprecated,
            29 => ErrorCode::RateLimitExceeded,
            _ => ErrorCode::Generic,
        };

        ErrorResponse {
            error: code,
            message: err.message,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct SimilarTracks {
    #[serde(rename = "similartracks")]
    similar_tracks: InnerSimilarTracks,
}

#[derive(Deserialize, Debug, PartialEq)]
struct InnerSimilarTracks {
    #[serde(rename = "track")]
    pub tracks: Vec<SimilarTrack>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Artist {
    pub name: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SimilarTrack {
    pub name: String,
    pub artist: Artist,
}

pub struct Client<'a> {
    api_key: &'a str,
    http: &'a hyper::Client<HttpConnector>,
}

impl<'a> Client<'a> {
    pub fn new(api_key: &'a str, client: &'a hyper::Client<HttpConnector>) -> Self {
        Client {
            api_key: api_key,
            http: client,
        }
    }

    pub fn similar_tracks(
        &self,
        artist: &str,
        track: &str,
    ) -> impl Future<Item = Vec<SimilarTrack>, Error = Error> {
        self.http
            .get(self.build_uri(
                "track.getsimilar",
                &format!("artist={}&track={}", artist, track),
            ))
            .map_err(Error::from)
            .and_then(parse_response)
            .map(|s: SimilarTracks| s.similar_tracks.tracks)
    }

    fn build_uri(&self, method: &str, params: &str) -> hyper::Uri {
        let endpoint = format!(
            "{}?method={}&api_key={}&format=json&{}",
            BASE_URL, method, self.api_key, params
        );

        endpoint.parse().unwrap()
    }
}

fn parse_response<T>(res: hyper::Response<hyper::Body>) -> impl Future<Item = T, Error = Error>
where
    for<'de> T: de::Deserialize<'de>,
{
    let (parts, body) = res.into_parts();

    body.concat2()
        .map_err(Error::from)
        .and_then(move |b| -> Result<T, Error> {
            if parts.status.is_success() {
                serde_json::from_slice(&b).map_err(Error::from)
            } else {
                serde_json::from_slice(&b)
                    .map_err(Error::from)
                    .and_then(|r: ProxyErrorResponse| Err(Error::APIError(r.into())))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn deserialize_similar_tracks() {
        let json = json!({"similartracks": {
            "track": [{
                "name": "Strong Enough",
                "playcount": 670120,
                "mbid": "39473218-db80-4db2-9623-690b79b94e04",
                "match": 1.0,
                "artist": {
                    "name": "Cher",
                    "mbid": "bfcc6d75-a6a5-4bc6-8282-47aec8531818"
                }
            }]
        }});

        let tracks: SimilarTracks = serde_json::from_value(json).unwrap();
        assert_eq!(
            tracks,
            SimilarTracks {
                similar_tracks: InnerSimilarTracks {
                    tracks: vec![SimilarTrack {
                        name: "Strong Enough".into(),
                        artist: Artist {
                            name: "Cher".into(),
                        },
                    }],
                },
            }
        );
    }

    #[test]
    fn deserialize_similar_tracks_api_error() {
        let json = json!({
            "error": 10,
            "message": "Invalid API Key"
        });

        let err: ProxyErrorResponse = serde_json::from_value(json).unwrap();
        assert_eq!(
            err,
            ProxyErrorResponse {
                error: 10,
                message: "Invalid API Key".into(),
            },
        );
    }
}
