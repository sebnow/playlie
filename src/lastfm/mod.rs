use hyper;
use hyper::client::HttpConnector;

static BASE_URL: &'static str = "http://ws.audioscrobbler.com/2.0";

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

    // TODO: Return a parsed structure
    pub fn similar_tracks(&self, artist: &str, track: &str) -> hyper::client::ResponseFuture {
        self.http.get(self.build_uri(
            "track.getsimilar",
            &format!("artist={}&track={}", artist, track),
        ))
    }

    fn build_uri(&self, method: &str, params: &str) -> hyper::Uri {
        let endpoint = format!(
            "{}?method={}&api_key={}&format=json&{}",
            BASE_URL, method, self.api_key, params
        );

        endpoint.parse().unwrap()
    }
}
