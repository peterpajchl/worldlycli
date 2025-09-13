use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64Standard};
use gcp_auth::{CustomServiceAccount, TokenProvider};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct TextToSpeechResponse {
    #[serde(rename = "audioContent")]
    audio_content: String,
    #[serde(rename = "audioConfig")]
    audio_config: AudioConfig,
}

#[derive(Debug, Deserialize)]
struct AudioConfig {
    #[serde(rename = "audioEncoding")]
    audio_encoding: String,
    //#[serde(rename = "speakingRate")]
    //speaking_rate: f32,
    //pitch: f32,
    //#[serde(rename = "volumeGainDb")]
    //volume_gain_db: f32,
    //#[serde(rename = "sampleRateHertz")]
    //sample_rate_hertz: u32,
}

pub(crate) async fn fetch_audio_for_text(client: Client, text: &str, dir: &str) -> Result<String> {
    let credentials_path = PathBuf::from("gcp-credentials.json");
    let service_account = CustomServiceAccount::from_file(credentials_path)?;
    let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
    let token = service_account.token(scopes).await?;
    let url = "https://texttospeech.googleapis.com/v1beta1/text:synthesize";

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .json(&json!({
            "input": {
                "text": text
            },
            "voice": {
                "languageCode": "en-GB",
                "name": "en-GB-Chirp-HD-O"
            },
            "audioConfig": {
                "audioEncoding": "MP3"
            }
        }))
        .send()
        .await?
        .json::<TextToSpeechResponse>()
        .await?;

    // make sure the output directory exists
    tokio::fs::create_dir_all(dir).await?;

    let mut hasher = DefaultHasher::new();
    let output_file = Base64Standard.decode(response.audio_content)?;
    text.hash(&mut hasher);
    let file_hash = format!("{}", hasher.finish());
    let file_name = format!(
        "{}.{}",
        file_hash,
        response.audio_config.audio_encoding.to_lowercase()
    );
    let file_path = Path::new(dir).join(file_name);
    tokio::fs::write(&file_path, &output_file).await?;

    Ok(file_path.to_str().unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_fetch() {
        let client = reqwest::Client::new();
        let result = match fetch_audio_for_text(client, "Hello, world!", "output/audio").await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error fetching audio: {}", e);
                Err(e)
            }
        };

        assert!(result.is_ok());
    }
}
