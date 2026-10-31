//! Bounded, device-independent passage framing. No silence-based session termination.
use super::audio::AudioFrame;
use std::collections::VecDeque;

pub enum PassageStep {
    Wait,
    Start(Vec<u8>),
    Append(Vec<u8>),
    End(Vec<u8>),
}

#[derive(Default)]
pub struct PassageGate {
    preroll: VecDeque<Vec<u8>>,
    frames: usize,
}

impl PassageGate {
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
            return Ok(PassageStep::Start(pcm));
        }
        self.frames += 1;
        if self.frames > 2000 {
            return Err("Passage trop long ; arrêt sans troncature silencieuse.");
        }
        if frame.end_passage {
            self.frames = 0;
            Ok(PassageStep::End(frame.pcm))
        } else {
            Ok(PassageStep::Append(frame.pcm))
        }
    }
}
