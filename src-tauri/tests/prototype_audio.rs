use vocalnotes_lib::platform::audio::AudioProcessor;

#[test]
fn downmix_and_resampling_are_streaming_bounded_and_mono_16k() {
    let mut audio = AudioProcessor::new(48_000, 2).unwrap();
    let mut output = Vec::new();
    // Two seconds of opposite stereo channels must downmix to silence.
    for _ in 0..200 {
        let samples: Vec<f32> = (0..480).flat_map(|_| [0.25, -0.25]).collect();
        output.extend(audio.push_interleaved(&samples).unwrap());
    }
    let samples: usize = output.iter().map(|f| f.pcm.len() / 2).sum();
    // Streaming resampler delay and the pending <30ms frame are retained, not invented as speech.
    assert!((30_000..=32_000).contains(&samples));
    assert!(
        output
            .iter()
            .all(|f| f.pcm.len() == 960 && !f.voiced && !f.end_passage)
    );
    assert!(output.iter().flat_map(|f| f.pcm.iter()).all(|b| *b == 0));
}

#[test]
fn mono_16k_is_not_resampled_and_silence_never_stops_session() {
    let mut audio = AudioProcessor::new(16_000, 1).unwrap();
    let frames = audio.push_interleaved(&vec![0.0; 480]).unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].pcm.len(), 960);
    assert!(!frames[0].end_passage);
    for _ in 0..100 {
        assert!(
            audio
                .push_interleaved(&vec![0.0; 480])
                .unwrap()
                .iter()
                .all(|f| !f.end_passage)
        );
    }
}

#[test]
fn bad_format_and_unbounded_input_fail_explicitly() {
    assert!(AudioProcessor::new(0, 0).is_err());
    let mut audio = AudioProcessor::new(16_000, 2).unwrap();
    assert!(audio.push_interleaved(&[0.0]).is_err());
    assert!(audio.push_interleaved(&[f32::NAN, 0.0]).is_err());
    assert!(audio.push_interleaved(&vec![0.0; 32_000]).is_err());
}
#[test]
fn openai_resampling_flushes_exact_duration_and_preserves_signal() {
    use vocalnotes_lib::audio::openai_pcm::OpenAiPcm;
    for frames in [1, 11, 101, 2000] {
        let mut converter = OpenAiPcm::new().unwrap();
        let pcm: Vec<u8> = (0..480)
            .flat_map(|i| {
                let value =
                    ((i as f32 * std::f32::consts::TAU * 440.0 / 16000.0).sin() * 12000.0) as i16;
                value.to_le_bytes()
            })
            .collect();
        let mut output = vec![];
        for _ in 0..frames {
            output.extend(converter.push(&pcm).unwrap());
        }
        output.extend(converter.finish().unwrap());
        assert_eq!(output.len(), frames * 480 * 3);
        assert!(
            output
                .chunks_exact(2)
                .any(|s| i16::from_le_bytes([s[0], s[1]]).unsigned_abs() > 10000)
        );
        assert!(converter.finish().is_err());
    }
    let mut converter = OpenAiPcm::new().unwrap();
    assert!(converter.push(&[0]).is_err());
}
