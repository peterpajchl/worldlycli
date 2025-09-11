use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

struct LatLong {
    lat: f64,
    long: f64,
}

#[derive(Debug, Deserialize)]
struct NominatimResponse {
    place_id: u32,
    licence: String,
    osm_type: String,
    osm_id: u32,
    lat: String,
    lon: String,
    class: String,
    #[serde(rename = "type")]
    place_type: String,
    place_rank: u32,
    importance: f32,
    addresstype: String,
    name: String,
    display_name: String,
    boundingbox: Vec<String>,
}

pub(crate) async fn get_lat_long_for_city(
    client: Client,
    city: &str,
    country: &str,
) -> Result<LatLong> {
    let response = client
        .get("https://nominatim.openstreetmap.org/search")
        .header("User-Agent", "worldlycli/0.1")
        .query(&[
            ("city", city),
            ("country", country),
            ("format", "json"),
            ("limit", "1"),
        ])
        .send()
        .await?
        .json::<Vec<NominatimResponse>>()
        .await?;

    let lat_long = LatLong {
        lat: response[0].lat.parse::<f64>()?,
        long: response[0].lon.parse::<f64>()?,
    };

    Ok(lat_long)
}
