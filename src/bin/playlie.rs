use tokio;
use reqwest;

use playlie::lastfm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api_key = std::env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY must be set");
    let http_client = reqwest::Client::new();
    let lfm = lastfm::Client::new(&api_key, &http_client);

    let res = lfm.user_recommended("sebnow").await?;

    for item in res.playlist {
        let artist: Vec<&str> = item.artists.iter().map(|a| a.name.as_str()).collect();
        println!("{} - {}", artist.join(" & "), item.name);
    }

    Ok(())
}
