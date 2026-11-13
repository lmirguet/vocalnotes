//! Bounded, device-independent passage framing. No silence-based session termination.
use super::audio::AudioFrame;
use std::collections::VecDeque;

pub enum PassageStep {
    Wait,
    Start {
        pcm: Vec<u8>,
        speech_started_at_epoch_us: Option<u64>,
    },
    Append(Vec<u8>),
    End {
        pcm: Vec<u8>,
        speech_ended_at_epoch_us: Option<u64>,
    },
}

#[derive(Default)]
pub struct PassageGate {
    preroll: VecDeque<Vec<u8>>,
    frames: usize,
    speech_started_at_epoch_us: Option<u64>,
    speech_ended_at_epoch_us: Option<u64>,
}

impl PassageGate {
    pub fn finish(&mut self) -> Option<PassageStep> {
        if self.frames == 0 {
            self.preroll.clear();
            return None;
        }
        self.frames = 0;
        let speech_ended_at_epoch_us = self.speech_ended_at_epoch_us;
        self.speech_started_at_epoch_us = None;
        self.speech_ended_at_epoch_us = None;
        Some(PassageStep::End {
            pcm: vec![0; 960],
            speech_ended_at_epoch_us,
        })
    }

    pub fn push(&mut self, frame: AudioFrame) -> Result<PassageStep, &'static str> {
        if frame.pcm.len() != 960 {
            return Err("Trame PCM invalide ; passage non intégré.");
        }
        if self.frames == 0 {
            if !frame.voiced {
                self.preroll.push_back(frame.pcm);
                if self.preroll.len() > 10 {
                    self.preroll.pop_front();
                }
                return Ok(PassageStep::Wait);
            }
            self.frames = self.preroll.len() + 1;
            let mut pcm: Vec<u8> = self.preroll.drain(..).flatten().collect();
            pcm.extend(frame.pcm);
            self.speech_started_at_epoch_us = frame
                .capture_start_epoch_us
                .map(|time| time.saturating_sub(30_000));
            self.speech_ended_at_epoch_us = frame.capture_start_epoch_us;
            return Ok(PassageStep::Start {
                pcm,
                speech_started_at_epoch_us: self.speech_started_at_epoch_us,
            });
        }
        if frame.voiced {
            self.speech_ended_at_epoch_us = frame.capture_start_epoch_us;
        }
        self.frames += 1;
        if self.frames > 2000 {
            return Err("Passage trop long ; arrêt sans troncature silencieuse.");
        }
        if frame.end_passage {
            self.frames = 0;
            let speech_ended_at_epoch_us = self.speech_ended_at_epoch_us;
            self.speech_started_at_epoch_us = None;
            self.speech_ended_at_epoch_us = None;
            Ok(PassageStep::End {
                pcm: frame.pcm,
                speech_ended_at_epoch_us,
            })
        } else {
            Ok(PassageStep::Append(frame.pcm))
        }
    }
}
