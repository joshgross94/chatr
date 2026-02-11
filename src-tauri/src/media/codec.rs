use tracing::error;

/// Opus encoder wrapper: 48kHz mono, 20ms frames (960 samples).
pub struct OpusEncoder {
    encoder: opus::Encoder,
}

impl OpusEncoder {
    pub fn new() -> Result<Self, String> {
        let encoder = opus::Encoder::new(48000, opus::Channels::Mono, opus::Application::Voip)
            .map_err(|e| format!("Failed to create Opus encoder: {}", e))?;
        Ok(Self { encoder })
    }

    /// Encode a 960-sample f32 PCM frame to Opus bytes.
    pub fn encode(&mut self, pcm: &[f32]) -> Result<Vec<u8>, String> {
        let mut output = vec![0u8; 4000]; // max opus frame
        let len = self
            .encoder
            .encode_float(pcm, &mut output)
            .map_err(|e| {
                error!("Opus encode error: {}", e);
                format!("Opus encode error: {}", e)
            })?;
        output.truncate(len);
        Ok(output)
    }
}

/// Opus decoder wrapper: 48kHz mono, 20ms frames (960 samples).
pub struct OpusDecoder {
    decoder: opus::Decoder,
}

impl OpusDecoder {
    pub fn new() -> Result<Self, String> {
        let decoder = opus::Decoder::new(48000, opus::Channels::Mono)
            .map_err(|e| format!("Failed to create Opus decoder: {}", e))?;
        Ok(Self { decoder })
    }

    /// Decode Opus bytes to a 960-sample f32 PCM frame.
    pub fn decode(&mut self, data: &[u8]) -> Result<Vec<f32>, String> {
        let mut output = vec![0.0f32; 960];
        let len = self
            .decoder
            .decode_float(data, &mut output, false)
            .map_err(|e| {
                error!("Opus decode error: {}", e);
                format!("Opus decode error: {}", e)
            })?;
        output.truncate(len);
        Ok(output)
    }
}
