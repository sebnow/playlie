use reqwest;
use serde::Deserialize;

pub mod errors;

static AS_BASE_URL: &'static str = "http://ws.audioscrobbler.com/2.0";
static LAST_FM_BASE_URL: &'static str = "https://last.fm";

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

#[derive(Deserialize, Debug, PartialEq)]
pub struct Playlist {
    pub playlist: Vec<PlaylistItem>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PlaylistItem {
    pub name: String,
    pub artists: Vec<Artist>,
}

pub struct Client<'a> {
    api_key: &'a str,
    http: &'a reqwest::Client,
}

impl<'a> Client<'a> {
    pub fn new(api_key: &'a str, client: &'a reqwest::Client) -> Self {
        Client {
            api_key,
            http: client,
        }
    }

    pub async fn similar_tracks(
        &self,
        artist: &str,
        track: &str,
    ) -> Result<Vec<SimilarTrack>, errors::Error> {
        let res = self
            .http
            .get(&self.build_as_uri(
                "track.getsimilar",
                &format!("artist={}&track={}", artist, track),
            ))
            .send()
            .await?
            .json::<SimilarTracks>().await?;

        Ok(res.similar_tracks.tracks)
    }

    pub async fn user_recommended(&self, user: &str) -> Result<Playlist, errors::Error> {
        let endpoint = format!("{}/player/station/user/{}/recommended", LAST_FM_BASE_URL, user);

        Ok(self.http.get(&endpoint).send().await?.json::<Playlist>().await?)
    }

    fn build_as_uri(&self, method: &str, params: &str) -> String {
        format!(
            "{}?method={}&api_key={}&format=json&{}",
            AS_BASE_URL, method, self.api_key, params
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
