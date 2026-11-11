use rubato::{FftFixedIn, Resampler};
use std::collections::VecDeque;

pub struct OpenAiPcm {
    resampler: FftFixedIn<f32>,
    pending: VecDeque<f32>,
    delay: usize,
    input_samples: usize,
    output_samples: usize,
    finished: bool,
}

impl OpenAiPcm {
    pub fn new() -> Result<Self, &'static str> {
        let resampler = FftFixedIn::new(16_000, 24_000, 480, 1, 1)
            .map_err(|_| "Conversion audio OpenAI indisponible.")?;
        let delay = resampler.output_delay();
        Ok(Self {
            resampler,
            pending: VecDeque::new(),
            delay,
            input_samples: 0,
            output_samples: 0,
            finished: false,
        })
    }

    fn block(&mut self, pad: bool, output: &mut Vec<u8>) -> Result<(), &'static str> {
        let n = self.resampler.input_frames_next();
        let mut input: Vec<f32> = self.pending.drain(..self.pending.len().min(n)).collect();
        if pad {
            input.resize(n, 0.0);
        }
        let result = self
            .resampler
            .process(&[input], None)
            .map_err(|_| "Conversion audio OpenAI impossible.")?;
        let target = self.input_samples * 3 / 2;
        for &sample in &result[0] {
            if self.delay > 0 {
                self.delay -= 1;
            } else if self.output_samples < target {
                output.extend_from_slice(
                    &((sample.clamp(-1.0, 1.0) * 32768.0)
                        .round()
                        .clamp(-32768.0, 32767.0) as i16)
                        .to_le_bytes(),
                );
                self.output_samples += 1;
            }
        }
        Ok(())
    }

    pub fn push(&mut self, pcm: &[u8]) -> Result<Vec<u8>, &'static str> {
        if self.finished
            || pcm.is_empty()
            || pcm.len() % 2 != 0
            || pcm.len() > 32_000
            || self.input_samples + pcm.len() / 2 > 16_000 * 60
        {
            self.finished = true;
            return Err("Format ou limite de conversion audio dépassé.");
        }
        self.input_samples += pcm.len() / 2;
        self.pending.extend(
            pcm.chunks_exact(2)
                .map(|s| f32::from(i16::from_le_bytes([s[0], s[1]])) / 32768.0),
        );
        let mut output = vec![];
        while self.pending.len() >= self.resampler.input_frames_next() {
            self.block(false, &mut output)?;
        }
        Ok(output)
    }

    pub fn finish(&mut self) -> Result<Vec<u8>, &'static str> {
        if self.finished || self.input_samples == 0 || self.input_samples % 2 != 0 {
            return Err("Conversion audio déjà terminée ou durée invalide.");
        }
        self.finished = true;
        let mut output = vec![];
        while self.output_samples < self.input_samples * 3 / 2 {
            self.block(true, &mut output)?;
        }
        Ok(output)
    }
}
