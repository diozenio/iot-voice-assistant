pub const SAMPLE_RATE: u32 = 16000;
pub const MIN_SECONDS: f64 = 0.5;

pub fn pcm_to_wav(pcm: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(44 + pcm.len());
    output.extend_from_slice(b"RIFF");
    output.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    output.extend_from_slice(b"WAVE");
    output.extend_from_slice(b"fmt ");
    output.extend_from_slice(&16u32.to_le_bytes());
    output.extend_from_slice(&1u16.to_le_bytes());
    output.extend_from_slice(&1u16.to_le_bytes());
    output.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    output.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    output.extend_from_slice(&2u16.to_le_bytes());
    output.extend_from_slice(&16u16.to_le_bytes());
    output.extend_from_slice(b"data");
    output.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    output.extend_from_slice(pcm);
    output
}

pub fn duration_seconds(pcm: &[u8]) -> f64 {
    pcm.len() as f64 / 2.0 / SAMPLE_RATE as f64
}

const DENOISE_RATE: usize = 48000;
const UPSAMPLE: usize = DENOISE_RATE / SAMPLE_RATE as usize;
const SPEECH_PROBABILITY: f32 = 0.5;

pub struct Enhanced {
    pub pcm: Vec<u8>,
    pub speech_frames: usize,
}

/// Suprime ruído com RNNoise (que opera a 48 kHz) e conta os quadros com fala.
pub fn enhance(pcm: &[u8]) -> Enhanced {
    let samples: Vec<f32> = pcm
        .chunks_exact(2)
        .map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]]) as f32)
        .collect();

    let mut upsampled = Vec::with_capacity(samples.len() * UPSAMPLE);
    for (index, &current) in samples.iter().enumerate() {
        let next = samples.get(index + 1).copied().unwrap_or(current);
        for step in 0..UPSAMPLE {
            upsampled.push(current + (next - current) * step as f32 / UPSAMPLE as f32);
        }
    }
    let padded = upsampled.len().div_ceil(nnnoiseless::FRAME_SIZE) * nnnoiseless::FRAME_SIZE;
    upsampled.resize(padded, 0.0);

    let mut state = nnnoiseless::DenoiseState::new();
    let mut cleaned = vec![0.0; upsampled.len()];
    let mut speech_frames = 0;
    for (index, (input, output)) in upsampled
        .chunks_exact(nnnoiseless::FRAME_SIZE)
        .zip(cleaned.chunks_exact_mut(nnnoiseless::FRAME_SIZE))
        .enumerate()
    {
        let probability = state.process_frame(output, input);
        // O primeiro quadro tem artefatos de inicialização do filtro.
        if index == 0 {
            output.fill(0.0);
        } else if probability >= SPEECH_PROBABILITY {
            speech_frames += 1;
        }
    }

    let output = cleaned
        .chunks_exact(UPSAMPLE)
        .take(samples.len())
        .flat_map(|group| {
            let mean = group.iter().sum::<f32>() / UPSAMPLE as f32;
            (mean.clamp(i16::MIN as f32, i16::MAX as f32) as i16).to_le_bytes()
        })
        .collect();

    Enhanced {
        pcm: output,
        speech_frames,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhance_keeps_length_and_ignores_silence() {
        let pcm = vec![0u8; SAMPLE_RATE as usize * 2];
        let enhanced = enhance(&pcm);
        assert_eq!(enhanced.pcm.len(), pcm.len());
        assert_eq!(enhanced.speech_frames, 0);
    }
}
