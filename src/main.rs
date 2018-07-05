extern crate hyper;
extern crate serde;
#[cfg_attr(test, macro_use)]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod lastfm;

use hyper::rt::{self, Future};

fn main() {
    let api_key = std::env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY must be set");
    let client = hyper::Client::new();
    let lfm = lastfm::Client::new(&api_key, &client);

    rt::run(lfm.similar_tracks("cher", "believe")
        .map(|res| {
            println!("Response: {:?}", res);
        })
        .map_err(|err| {
            println!("Error: {:?}", err);
        }));
}
