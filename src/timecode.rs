use oximedia::timecode::{
    FrameRate, Timecode, TimecodeError, TimecodeReader,
    ltc::{LtcReader, LtcReaderConfig},
};
use rodio::microphone::{Input, InputConfig, MicrophoneBuilder};
use std::{io::Read, sync::mpsc, thread};

pub type TcBlob = Result<(f32, Timecode), TimecodeError>;

pub fn start_timecode_listen(
    input_device: Input,
    frame_rate: FrameRate,
) -> Option<mpsc::Receiver<TcBlob>> {
    let (snd, rec) = mpsc::channel();

    let config = InputConfig::default();
    let mut input = MicrophoneBuilder::new()
        .device(input_device)
        .ok()?
        .config(config)
        .ok()?
        .open_stream()
        .ok()?;

    thread::spawn(move || {
        let ltc_config = LtcReaderConfig {
            frame_rate,
            max_speed: 2.0,
            min_amplitude: 1e-3,
            sample_rate: config.sample_rate.into(),
        };

        let mut tc_decoder = LtcReader::new(ltc_config);

        loop {
            let chunk = input.by_ref().take(256).collect::<Vec<f32>>();
            let res = tc_decoder.process_samples(&chunk);

            if chunk.is_empty() {
                break;
            }

            //for mut time in res {
            if let Ok(Some(mut time)) = res {
                // to get the CURRENT frame, we guess that time is linear,
                // because we can't read the current frame until it's ended.
                let _ = time.increment();
                if snd.send(Ok((tc_decoder.sync_confidence(), time))).is_err() {
                    break;
                }
            } else if let Err(e) = res {
                println!("error: {e}");
                let _ = snd.send(Err(e));
            }
        }
    });

    Some(rec)
}

mod tests {
    use super::*;

    #[test]
    fn ltc_decode_from_file() {
        let mut wav_reader = hound::WavReader::open("smpte_25fps.wav").unwrap();

        let frame_rate = FrameRate::Fps25;

        let sample_rate = wav_reader.spec().sample_rate;
        let ltc_config = LtcReaderConfig {
            frame_rate,
            max_speed: 2.0,
            min_amplitude: 1e-3,
            sample_rate,
        };
        let mut tc_decoder = LtcReader::new(ltc_config);

        let mut reader = wav_reader.samples::<i16>();

        let mut timestamps = vec![];

        loop {
            let chunk = reader
                .by_ref()
                .take(256)
                .map(|s| s.unwrap() as f32 / 32767.0 * -1.5)
                .collect::<Vec<f32>>();
            let res = tc_decoder.process_samples(&chunk);
            let peak = chunk.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
            if let Ok(Some(time)) = res {
                timestamps.push(time);
            }

            if chunk.is_empty() {
                break;
            }
        }
        assert!(!timestamps.is_empty());
        for i in 3..timestamps.len() - 1 {
            assert!(
                timestamps[i] <= timestamps[i + 1],
                "{} and {} are misordered",
                timestamps[i],
                timestamps[i + 1]
            )
        }
    }
}
