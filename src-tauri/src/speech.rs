use std::f64::consts::PI;

use crate::audio::SAMPLE_RATE;

const MODELS: &str = "https://openrouter.ai/api/v1/models?output_modalities=speech";
const SPEECH: &str = "https://openrouter.ai/api/v1/audio/speech";
const DEFAULT_TTS_RATE: u32 = 24000;
const HALF_TAPS: i64 = 16;

pub async fn voices(http: &reqwest::Client, model: &str) -> Result<Vec<String>, String> {
    let response = http.get(MODELS).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("OpenRouter {}", response.status().as_u16()));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let voices = body["data"]
        .as_array()
        .and_then(|models| models.iter().find(|entry| entry["id"] == model))
        .ok_or_else(|| format!("Modelo de voz '{model}' não encontrado"))?["supported_voices"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|voice| voice.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    Ok(voices)
}

pub async fn synthesize(
    http: &reqwest::Client,
    api_key: &str,
    model: &str,
    voice: &str,
    instructions: &str,
    text: &str,
) -> Result<Vec<i16>, String> {
    let mut body = serde_json::json!({
        "model": model,
        "input": text,
        "voice": voice,
        "response_format": "pcm",
    });
    if !instructions.trim().is_empty() {
        body["instructions"] = instructions.trim().into();
    }

    let response = http
        .post(SPEECH)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenRouter TTS {}: {body}", status.as_u16()));
    }

    let rate = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .split(';')
                .find_map(|part| part.trim().strip_prefix("rate="))
        })
        .and_then(|rate| rate.parse().ok())
        .unwrap_or(DEFAULT_TTS_RATE);

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    let samples: Vec<i16> = bytes
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    if samples.is_empty() {
        return Err("A OpenRouter devolveu áudio vazio".into());
    }

    Ok(resample(&samples, rate, SAMPLE_RATE))
}

pub fn resample(input: &[i16], from: u32, to: u32) -> Vec<i16> {
    if from == to {
        return input.to_vec();
    }
    let ratio = from as f64 / to as f64;
    let cutoff = (1.0 / ratio).min(1.0) * 0.95;
    let length = (input.len() as f64 / ratio) as usize;

    (0..length)
        .map(|n| {
            let center = n as f64 * ratio;
            let first = center.floor() as i64 - HALF_TAPS + 1;
            let mut sum = 0.0;
            let mut weight = 0.0;
            for k in first..first + 2 * HALF_TAPS {
                let t = k as f64 - center;
                let x = PI * t * cutoff;
                let sinc = if x.abs() < 1e-9 { 1.0 } else { x.sin() / x };
                let window = 0.5 + 0.5 * (PI * t / HALF_TAPS as f64).cos();
                let tap = cutoff * sinc * window;
                weight += tap;
                if let Some(sample) = usize::try_from(k).ok().and_then(|k| input.get(k)) {
                    sum += tap * *sample as f64;
                }
            }
            (sum / weight)
                .round()
                .clamp(i16::MIN as f64, i16::MAX as f64) as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_changes_length_and_keeps_level() {
        let input = vec![8000i16; 2400];
        let output = resample(&input, 24000, 16000);
        assert_eq!(output.len(), 1600);
        assert!((output[800] as i32 - 8000).abs() < 50);
    }
}
