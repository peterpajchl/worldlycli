use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64Standard};
use gcp_auth::{CustomServiceAccount, TokenProvider};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;

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
    #[serde(rename = "speakingRate")]
    speaking_rate: f32,
    pitch: f32,
    #[serde(rename = "volumeGainDb")]
    volume_gain_db: f32,
    #[serde(rename = "sampleRateHertz")]
    sample_rate_hertz: u32,
}

pub(crate) async fn fetch_audio_for_text(client: Client, text: &str) -> Result<()> {
    let credentials_path = PathBuf::from("worldly-471715-10ed2befe42a.json");
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

    dbg!(&response);

    let output_file = Base64Standard.decode(response.audio_content)?;
    tokio::fs::write("output.mp3", &output_file).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_fetch() {
        let client = reqwest::Client::new();
        let result = match fetch_audio_for_text(client, "Hello, world!").await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error fetching audio: {}", e);
                Err(e)
            }
        };

        assert!(result.is_ok());
    }
}
