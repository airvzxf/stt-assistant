#[allow(dead_code)]
pub struct Vad {
    threshold: f32,
    hold_frames: usize,
    silence_counter: usize,
    is_speaking: bool,
    sample_rate: u32,
}

#[allow(dead_code)]
impl Vad {
    pub fn new(threshold: f32, hold_ms: u64, sample_rate: u32) -> Self {
        Self {
            threshold,
            hold_frames: (hold_ms as f32 / 20.0) as usize, // Assuming ~20ms chunks logic roughly
            silence_counter: 0,
            is_speaking: false,
            sample_rate,
        }
    }

    // Simple RMS calculation
    fn calculate_rms(samples: &[f32]) -> f32 {
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    pub fn is_voice_segment(&mut self, samples: &[f32]) -> bool {
        let rms = Vad::calculate_rms(samples);

        if rms > self.threshold {
            self.is_speaking = true;
            self.silence_counter = 0;
        } else if self.is_speaking {
            self.silence_counter += 1;
            if self.silence_counter > 20 {
                // 20 * chunk_duration wait
                self.is_speaking = false;
            }
        }

        self.is_speaking
    }
}
