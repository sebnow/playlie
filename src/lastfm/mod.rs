use hyper;
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use serde_json;

static BASE_URL: &'static str = "http://ws.audioscrobbler.com/2.0";

#[derive(Debug)]
pub enum Error {
    /// An error occurred while parsing the response
    ParsingError(serde_json::error::Error),
    /// An error occurred during the request
    HTTPError(hyper::error::Error),
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
            .and_then(|r| r.into_body().concat2())
            .map_err(|e| Error::HTTPError(e))
            .and_then(|body| serde_json::from_slice(&body).map_err(Error::ParsingError))
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
}