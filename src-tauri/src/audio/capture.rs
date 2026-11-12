//! Native CPAL capture. Device opening is explicit and lifecycle guarded.
use super::normalization::{AudioFrame, AudioMeter, AudioProcessor};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rtrb::{Producer, RingBuffer};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

// 0 stopped, 1 running, 2 device error, 3 saturation, 4 normalization error.
pub struct NativeCapture {
    stop: Arc<AtomicBool>,
    status: Arc<AtomicU8>,
    worker: Option<JoinHandle<()>>,
    pub frames: Receiver<AudioFrame>,
}

#[derive(Clone)]
pub struct CaptureControl {
    stop: Arc<AtomicBool>,
    status: Arc<AtomicU8>,
}

impl CaptureControl {
    pub fn stop(&self) {
        // Callback stops accepting samples immediately; worker drops the stream.
        self.status.store(0, Ordering::Release);
        self.stop.store(true, Ordering::Release);
    }
}

struct CaptureMarker {
    sample_count: usize,
    consumed: usize,
    capture_at_epoch_us: Option<u64>,
}

fn capture_epoch_us_from_callback(callback_epoch_us: u64, capture_age: Duration) -> Option<u64> {
    let age_us = u64::try_from(capture_age.as_micros()).ok()?;
    callback_epoch_us.checked_sub(age_us)
}

fn current_epoch_us() -> Option<u64> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_micros(),
    )
    .ok()
}

fn capture_timestamp(info: &cpal::InputCallbackInfo) -> Option<u64> {
    let timestamp = info.timestamp();
    let capture_age = timestamp.callback.duration_since(&timestamp.capture)?;
    capture_epoch_us_from_callback(current_epoch_us()?, capture_age)
}

fn push_samples<T>(
    producer: &mut Producer<f32>,
    status: &AtomicU8,
    stop: &AtomicBool,
    samples: &[T],
) -> bool
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    if stop.load(Ordering::Acquire) || status.load(Ordering::Acquire) != 1 {
        return false;
    }
    for &sample in samples {
        if producer
            .push(cpal::Sample::to_sample::<f32>(sample))
            .is_err()
        {
            status.store(3, Ordering::Release);
            return false;
        }
    }
    true
}

fn stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut producer: Producer<f32>,
    mut markers: Producer<CaptureMarker>,
    status: Arc<AtomicU8>,
    stop: Arc<AtomicBool>,
) -> Result<cpal::Stream, &'static str>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let callback_status = status.clone();
    device
        .build_input_stream(
            config,
            move |samples: &[T], info| {
                // No allocation, downmix, resampling, locks, network, logging or disk here.
                if samples.is_empty()
                    || !push_samples(&mut producer, &callback_status, &stop, samples)
                {
                    return;
                }
                if markers
                    .push(CaptureMarker {
                        sample_count: samples.len(),
                        consumed: 0,
                        capture_at_epoch_us: capture_timestamp(&info),
                    })
                    .is_err()
                {
                    callback_status.store(3, Ordering::Release);
                }
            },
            move |_| {
                status.store(2, Ordering::Release);
            },
            None,
        )
        .map_err(|_| "Ouverture du microphone impossible.")
}

impl NativeCapture {
    pub fn control(&self) -> CaptureControl {
        CaptureControl {
            stop: self.stop.clone(),
            status: self.status.clone(),
        }
    }
    /// Call ONLY after explicit consent, account selection and verified OS lifecycle guard.
    /// Nothing in app startup calls this method.
    pub fn start_explicit() -> Result<Self, &'static str> {
        Self::start_guarded(Arc::new(AtomicBool::new(false)))
    }

    pub fn start_guarded(stop: Arc<AtomicBool>) -> Result<Self, &'static str> {
        if stop.load(Ordering::Acquire) {
            return Err("Initialisation du microphone annulée.");
        }
        let status = Arc::new(AtomicU8::new(1));
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (frames_tx, frames_rx) = mpsc::sync_channel(64); // <2 seconds at 30ms.
        let worker_stop = stop.clone();
        let worker_status = status.clone();
        let worker = thread::spawn(move || {
            let result = run(worker_stop, worker_status.clone(), frames_tx, &ready_tx);
            if let Err(message) = result {
                let _ = ready_tx.try_send(Err(message));
                if worker_status.load(Ordering::Acquire) == 1 {
                    worker_status.store(2, Ordering::Release);
                }
            }
        });
        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Self {
                stop,
                status,
                worker: Some(worker),
                frames: frames_rx,
            }),
            result => {
                stop.store(true, Ordering::Release);
                let _ = worker.join();
                Err(result
                    .ok()
                    .and_then(Result::err)
                    .unwrap_or("Initialisation du microphone expirée."))
            }
        }
    }

    pub fn failure(&self) -> Option<&'static str> {
        match self.status.load(Ordering::Acquire) {
            2 => Some("Microphone indisponible : arrêt requis, passage non intégré."),
            3 => Some("Saturation audio : arrêt requis, aucune perte silencieuse ni spool disque."),
            4 => Some("Normalisation audio échouée : arrêt requis, passage non intégré."),
            _ => None,
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for NativeCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(
    stop: Arc<AtomicBool>,
    status: Arc<AtomicU8>,
    frames: SyncSender<AudioFrame>,
    ready: &SyncSender<Result<(), &'static str>>,
) -> Result<(), &'static str> {
    if stop.load(Ordering::Acquire) {
        return Err("Capture annulée avant ouverture.");
    }
    let device = cpal::default_host()
        .default_input_device()
        .ok_or("Aucun microphone natif disponible.")?;
    let supported = device
        .default_input_config()
        .map_err(|_| "Configuration du microphone indisponible.")?;
    let config: cpal::StreamConfig = supported.clone().into();
    let rate = config.sample_rate.0 as usize;
    let channels = config.channels as usize;
    crate::diagnostics::numbers(
        "Microphone : configuration entrée native par défaut",
        &[("sample_rate", rate as u64), ("channels", channels as u64)],
    );
    crate::diagnostics::event(match device.name().as_deref() {
        Ok("default") => "Route CPAL : default",
        Ok("pipewire") => "Route CPAL : pipewire",
        Ok("pulse") => "Route CPAL : pulse",
        _ => "Route CPAL : autre périphérique natif par défaut (nom masqué)",
    });
    let mut processor = AudioProcessor::new(rate, channels)?;
    let (producer, mut consumer) = RingBuffer::<f32>::new(rate * channels * 2);
    let (marker_producer, mut marker_consumer) = RingBuffer::<CaptureMarker>::new(1024);
    let capture = match supported.sample_format() {
        cpal::SampleFormat::F32 => stream::<f32>(
            &device,
            &config,
            producer,
            marker_producer,
            status.clone(),
            stop.clone(),
        )?,
        cpal::SampleFormat::I16 => stream::<i16>(
            &device,
            &config,
            producer,
            marker_producer,
            status.clone(),
            stop.clone(),
        )?,
        cpal::SampleFormat::U16 => stream::<u16>(
            &device,
            &config,
            producer,
            marker_producer,
            status.clone(),
            stop.clone(),
        )?,
        _ => return Err("Format du microphone non pris en charge ; aucune capture."),
    };
    if stop.load(Ordering::Acquire) {
        return Err("Capture annulée avant démarrage.");
    }
    capture
        .play()
        .map_err(|_| "Démarrage du microphone refusé.")?;
    ready.send(Ok(())).map_err(|_| "Capture annulée.")?;
    let chunk = rate / 100 * channels;
    let mut scratch = Vec::with_capacity(chunk);
    let mut scratch_capture_at_epoch_us = None;
    let mut current_marker: Option<CaptureMarker> = None;
    let mut meter = AudioMeter::default();
    let mut reported = Instant::now();
    while !stop.load(Ordering::Acquire) && status.load(Ordering::Acquire) == 1 {
        while scratch.len() < chunk {
            if current_marker
                .as_ref()
                .is_none_or(|marker| marker.consumed == marker.sample_count)
            {
                current_marker = marker_consumer.pop().ok();
            }
            let Some(marker) = current_marker.as_mut() else {
                break;
            };
            match consumer.pop() {
                Ok(sample) => {
                    if scratch.is_empty() {
                        scratch_capture_at_epoch_us = marker.capture_at_epoch_us.map(|start| {
                            start.saturating_add(
                                (marker.consumed as u64 / channels as u64 * 1_000_000)
                                    / rate as u64,
                            )
                        });
                    }
                    marker.consumed += 1;
                    scratch.push(sample);
                }
                Err(_) => break,
            }
        }
        if scratch.len() == chunk {
            meter.observe(&scratch);
            match processor.push_interleaved_timed(&scratch, scratch_capture_at_epoch_us) {
                Ok(packets) => {
                    for packet in packets {
                        meter.frame(&packet);
                        if frames.try_send(packet).is_err() {
                            status.store(3, Ordering::Release);
                            break;
                        }
                    }
                }
                Err(_) => status.store(4, Ordering::Release),
            }
            scratch.clear();
            scratch_capture_at_epoch_us = None;
        } else {
            thread::sleep(Duration::from_millis(2));
        }
        if reported.elapsed() >= Duration::from_secs(1) {
            let (samples, rms, peak) = meter.levels();
            crate::diagnostics::numbers(
                "Audio reçu : fenêtre de mesure",
                &[
                    ("input_samples", samples),
                    ("rms_ppm", (rms * 1_000_000.0).round() as u64),
                    ("peak_ppm", (f64::from(peak) * 1_000_000.0).round() as u64),
                    (
                        "pcm_peak_ppm",
                        u64::from(meter.pcm_peak) * 1_000_000 / 32768,
                    ),
                    ("frames_30ms", meter.frames),
                    ("voiced_frames", meter.voiced),
                ],
            );
            meter = AudioMeter::default();
            reported = Instant::now();
        }
    }
    drop(capture); // Stop device BEFORE publishing completion.
    crate::diagnostics::event("Microphone : flux natif fermé");
    if status.load(Ordering::Acquire) == 1 {
        status.store(0, Ordering::Release);
    }
    Ok(())
}

#[cfg(test)]
mod ingress_tests {
    use super::*;
    #[test]
    fn capture_clock_mapping_subtracts_callback_age_and_rejects_underflow() {
        assert_eq!(
            capture_epoch_us_from_callback(10_000_000, Duration::from_micros(1_250)),
            Some(9_998_750)
        );
        assert_eq!(
            capture_epoch_us_from_callback(1_000, Duration::from_micros(1_250)),
            None
        );
    }

    #[test]
    fn saturated_callback_is_terminal_and_never_overwrites_queued_samples() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(2);
        let status = AtomicU8::new(1);
        let stop = AtomicBool::new(false);
        assert!(!push_samples(
            &mut producer,
            &status,
            &stop,
            &[0.1f32, 0.2, 0.3]
        ));
        assert_eq!(status.load(Ordering::Acquire), 3);
        assert_eq!(consumer.pop().unwrap(), 0.1);
        assert_eq!(consumer.pop().unwrap(), 0.2);
        assert!(!push_samples(&mut producer, &status, &stop, &[0.4f32]));
        assert!(consumer.pop().is_err());
    }
    #[test]
    fn stopped_callback_accepts_no_sample_without_opening_a_device() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(2);
        let status = AtomicU8::new(1);
        let stop = AtomicBool::new(true);
        assert!(!push_samples(&mut producer, &status, &stop, &[0.1f32]));
        assert!(consumer.pop().is_err());
    }
}
