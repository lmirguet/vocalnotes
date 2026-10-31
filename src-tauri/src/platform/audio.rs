//! Streaming normalization outside the real-time callback. No device is opened here.
use rubato::{FftFixedIn, Resampler};
use std::collections::VecDeque;
use webrtc_vad::{SampleRate, Vad, VadMode};

const OUTPUT_RATE: usize = 16_000;
const FRAME: usize = 480; // WebRTC-valid 30 ms, engineering parameter, not a measured SLA.
const SILENCE_FRAMES: usize = 27; // 810ms, to calibrate on the two target desktops.
const MAX_PASSAGE_FRAMES: usize = 2000; // 60s bound: explicit failure, never silent truncation.

pub struct AudioFrame {
    pub pcm: Vec<u8>,
    pub voiced: bool,
    pub end_passage: bool,
}

pub struct AudioProcessor {
    rate: usize,
    channels: usize,
    input: VecDeque<f32>,
    output: VecDeque<i16>,
    resampler: Option<FftFixedIn<f32>>,
    delay: usize,
    vad: Vad,
    active: bool,
    silence: usize,
    passage: usize,
}

impl AudioProcessor {
    pub fn new(rate: usize, channels: usize) -> Result<Self, &'static str> {
        if !(8000..=192_000).contains(&rate) || !(1..=8).contains(&channels) {
            return Err("Format audio natif non pris en charge.");
        }
        let resampler = if rate == OUTPUT_RATE {
            None
        } else {
            Some(
                FftFixedIn::<f32>::new(rate, OUTPUT_RATE, rate / 100, 1, 1)
                    .map_err(|_| "Normalisation audio indisponible.")?,
            )
        };
        let delay = resampler.as_ref().map_or(0, |r| r.output_delay());
        Ok(Self {
            rate,
            channels,
            input: VecDeque::new(),
            output: VecDeque::new(),
            resampler,
            delay,
            vad: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Quality),
            active: false,
            silence: 0,
            passage: 0,
        })
    }

    fn normalized(&mut self, mono: &[f32]) {
        for &sample in mono {
            if self.delay > 0 {
                self.delay -= 1;
                continue;
            }
            self.output
                .push_back((sample.clamp(-1.0, 1.0) * 32767.0).round() as i16);
        }
    }

    pub fn push_interleaved(&mut self, samples: &[f32]) -> Result<Vec<AudioFrame>, &'static str> {
        if samples.len() % self.channels != 0
            || samples.len() > self.rate * self.channels / 4
            || samples.iter().any(|s| !s.is_finite())
        {
            return Err("Entrée audio invalide ou trop volumineuse ; capture à arrêter.");
        }
        for frame in samples.chunks_exact(self.channels) {
            self.input
                .push_back(frame.iter().sum::<f32>() / self.channels as f32);
        }
        if self.resampler.is_some() {
            loop {
                let needed = self.resampler.as_ref().unwrap().input_frames_next();
                if self.input.len() < needed {
                    break;
                }
                let mono: Vec<f32> = self.input.drain(..needed).collect();
                let converted = self
                    .resampler
                    .as_mut()
                    .unwrap()
                    .process(&[mono], None)
                    .map_err(|_| "Normalisation audio échouée ; passage non intégré.")?;
                self.normalized(&converted[0]);
            }
        } else {
            let mono: Vec<f32> = self.input.drain(..).collect();
            self.normalized(&mono);
        }
        let mut frames = Vec::new();
        while self.output.len() >= FRAME {
            let samples: Vec<i16> = self.output.drain(..FRAME).collect();
            let voiced = self
                .vad
                .is_voice_segment(&samples)
                .map_err(|_| "VAD local indisponible.")?;
            if voiced {
                self.active = true;
                self.silence = 0;
            } else if self.active {
                self.silence += 1;
            }
            if self.active {
                self.passage += 1;
            }
            if self.passage > MAX_PASSAGE_FRAMES {
                return Err("Passage trop long ; arrêt requis, aucune troncature silencieuse.");
            }
            let end_passage = self.active && self.silence >= SILENCE_FRAMES;
            if end_passage {
                self.active = false;
                self.silence = 0;
                self.passage = 0;
            }
            frames.push(AudioFrame {
                pcm: samples.iter().flat_map(|s| s.to_le_bytes()).collect(),
                voiced,
                end_passage,
            });
        }
        Ok(frames)
    }
}
