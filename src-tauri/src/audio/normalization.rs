//! Streaming normalization outside the real-time callback. No device is opened here.
use rubato::{FftFixedIn, Resampler};
use std::collections::VecDeque;
use webrtc_vad::{SampleRate, Vad, VadMode};

const OUTPUT_RATE: usize = 16_000;
const FRAME: usize = 480; // WebRTC-valid 30 ms, engineering parameter, not a measured SLA.
const SILENCE_FRAMES: usize = 27; // 810ms, to calibrate on the two target desktops.
const MAX_PASSAGE_FRAMES: usize = 2000; // 60s bound: explicit failure, never silent truncation.

/// Numeric diagnostics only; never retains a sample or transcript.
#[derive(Default)]
pub(crate) struct AudioMeter {
    samples: u64,
    squares: f64,
    peak: f32,
    pub frames: u64,
    pub voiced: u64,
    pub pcm_peak: u32,
}

impl AudioMeter {
    pub fn frame(&mut self, frame: &AudioFrame) {
        self.frames += 1;
        self.voiced += u64::from(frame.voiced);
        for sample in frame.pcm.chunks_exact(2) {
            self.pcm_peak = self.pcm_peak.max(u32::from(
                i16::from_le_bytes([sample[0], sample[1]]).unsigned_abs(),
            ));
        }
    }
    pub fn observe(&mut self, samples: &[f32]) {
        for &sample in samples {
            if sample.is_finite() {
                self.samples += 1;
                self.squares += f64::from(sample).powi(2);
                self.peak = self.peak.max(sample.abs());
            }
        }
    }

    pub fn levels(&self) -> (u64, f64, f32) {
        let rms = if self.samples == 0 {
            0.0
        } else {
            (self.squares / self.samples as f64).sqrt()
        };
        (self.samples, rms, self.peak)
    }
}

#[cfg(test)]
mod meter_tests {
    use super::*;

    #[test]
    fn meter_distinguishes_silence_signal_and_no_input_without_channel_cancellation() {
        let mut meter = AudioMeter::default();
        assert_eq!(meter.levels(), (0, 0.0, 0.0));
        meter.observe(&[0.0; 480]);
        assert_eq!(meter.levels(), (480, 0.0, 0.0));
        let mut stereo = AudioMeter::default();
        stereo.observe(&[0.25, -0.25, 0.25, -0.25]);
        assert_eq!(stereo.levels(), (4, 0.25, 0.25));
        let mut audio = AudioProcessor::new(16_000, 2).unwrap();
        let input: Vec<f32> = (0..480).flat_map(|_| [0.25, -0.25]).collect();
        let frames = audio.push_interleaved(&input).unwrap();
        assert!(frames.iter().all(|f| f.pcm.iter().all(|b| *b == 0)));
        for frame in &frames {
            stereo.frame(frame);
        }
        assert_eq!(stereo.frames, 1);
        assert_eq!(stereo.voiced, 0);
        assert_eq!(stereo.pcm_peak, 0);
        stereo.frame(&AudioFrame {
            pcm: i16::MIN.to_le_bytes().to_vec(),
            voiced: true,
            end_passage: false,
        });
        assert_eq!(stereo.pcm_peak, 32768);
        assert_eq!(stereo.voiced, 1);
    }
}

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
