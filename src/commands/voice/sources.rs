use std::time::Duration;

struct ReturnSource {
    title: String,
    artist: String, 
    source_url: String,
    thumbnail: String,
    duration: Duration
}


async fn audius_track(artist: String, slug: String, id: i64) {
const AUDIUS_PROVIDER: &str = "https://dn-usa.audius.metadata.fyi/";  // will select from api.audius.co later?
let client = reqwest::Client::new();
}