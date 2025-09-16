mod audio_service;
mod latlong_service;

use audio_service::fetch_audio_for_text;
use csv::ReaderBuilder;
use latlong_service::LatLong;
use latlong_service::get_lat_long_for_city;
use reqwest::Client;
use serde::de;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Deserialize, Serialize)]
struct CountryCapital {
    #[serde(alias = "SHORT_FORM_NAME")]
    country_short_form_name: String,
    #[serde(alias = "LONG_FORM_NAME")]
    country_long_form_name: String,
    #[serde(alias = "GENC_2A_CODE")]
    country_code_2letter: String,
    #[serde(alias = "GENC_3A_CODE")]
    country_code_3letter: String,
    #[serde(alias = "CAPITAL_INDEPENDENT_STATES")]
    capital_city: String,
    #[serde(alias = "STATUS")]
    #[serde(deserialize_with = "deserialize_independent_to_bool")]
    independent: bool,
    #[serde(alias = "MEMBER_OF_UNITED_NATIONS")]
    #[serde(deserialize_with = "deserialize_member_of_un_to_bool")]
    member_of_un: bool,
    capital_latitude: Option<f64>,
    capital_longitude: Option<f64>,
    country_audio_filename: Option<String>,
    capital_audio_filename: Option<String>,
}

impl CountryCapital {
    fn set_latlong(&mut self, lat_long: LatLong) {
        self.capital_latitude = Some(lat_long.lat);
        self.capital_longitude = Some(lat_long.long);
    }

    fn set_audio_filenames(&mut self, country_audio: String, capital_audio: String) {
        self.country_audio_filename = Some(country_audio);
        self.capital_audio_filename = Some(capital_audio);
    }
}

fn deserialize_member_of_un_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "TRUE" => Ok(true),
        _ => Ok(false),
    }
}

fn deserialize_independent_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "Independent" => Ok(true),
        _ => Ok(false),
    }
}

const OUTPUT_PATH: &str = "output/audio";

pub async fn run() -> anyhow::Result<()> {
    // 1. load the csv file and parse it
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_path("independent-countries.csv")?;

    let http_client = Client::new();
    let file_handle = tokio::fs::File::create("output.json").await?;
    let mut json_writer = tokio::io::BufWriter::new(file_handle);

    json_writer.write_all(b"[\n").await?;

    let mut csv_iter = csv_reader.deserialize::<CountryCapital>().peekable();

    while let Some(entry) = csv_iter.next() {
        let is_last = csv_iter.peek().is_none();

        match entry {
            Ok(mut r) => {
                println!(
                    "Country: '{}' Capital: '{}'",
                    r.country_short_form_name, r.capital_city
                );
                let lat_long = get_lat_long_for_city(
                    http_client.clone(),
                    &r.capital_city,
                    &r.country_code_2letter,
                )
                .await?;

                r.set_latlong(lat_long);

                let country_audio_resp = fetch_audio_for_text(
                    http_client.clone(),
                    &r.country_short_form_name,
                    OUTPUT_PATH,
                )
                .await?;

                let capital_audio_resp =
                    fetch_audio_for_text(http_client.clone(), &r.capital_city, OUTPUT_PATH).await?;

                r.set_audio_filenames(country_audio_resp, capital_audio_resp);

                let json_row = serde_json::to_string(&r)?;
                json_writer.write_all(json_row.as_bytes()).await?;

                if !is_last {
                    json_writer.write_all(b",\n").await?;
                }
            }
            Err(e) => eprintln!("Error parsing record: {}", e),
        };
    }

    // 2. run through the list and fetch
    // a) the lat and long for each capital city
    // b) generate audio using text-to-speech for each capital city and country name
    //
    // create new json file with all the data
    json_writer.write_all(b"\n]\n").await?;
    json_writer.flush().await?;

    Ok(())
}
