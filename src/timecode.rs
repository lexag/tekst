use crate::ltc::readwrite::{LtcReader, LtcReaderConfig};
use crate::ltc::{FrameRate, Timecode, TimecodeError};
use rodio::microphone::{Input, InputConfig, MicrophoneBuilder};
use rodio::{DeviceTrait, microphone::Microphone};
use std::{
    ops::Sub,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

type TcBlob = Result<(f32, Timecode), TimecodeError>;

pub struct TimecodeReader {
    listening_thread: Option<JoinHandle<()>>,
    confidence: f32,
    last_timecode: Option<Timecode>,
    frame_rate: FrameRate,
    recv: Receiver<TcBlob>,
    send: Sender<TcBlob>,
    available_devices: Vec<String>,
    selected_device_idx: Option<usize>,
}

impl TimecodeReader {
    pub fn new() -> Self {
        let (snd, rec) = mpsc::channel();
        let mut a = Self {
            available_devices: vec![],
            listening_thread: None,
            selected_device_idx: None,
            confidence: 0.0,
            last_timecode: None,
            frame_rate: FrameRate::Fps25,
            recv: rec,
            send: snd,
        };
        a.reload_available_devices();
        a
    }

    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    pub fn update(&mut self) -> Result<(), TimecodeError> {
        self.confidence = self.confidence.sub(0.01).max(0.0);
        while let Ok(res) = self.recv.try_recv() {
            match res {
                Ok((confidence, time)) => {
                    self.confidence = confidence;
                    self.last_timecode = Some(time);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn timecode(&self) -> Option<Timecode> {
        let mut tc = self.last_timecode?;
        tc.increment().ok()?;
        Some(tc)
    }

    pub fn reload_available_devices(&mut self) {
        if let Ok(devices) = rodio::microphone::available_inputs() {
            for device in devices {
                if let Ok(desc) = device.clone().into_inner().description() {
                    let device_string =
                        format!("{} ({})", desc.name(), desc.extended().join(" | "));
                    self.available_devices.push(device_string);
                }
            }
        }
    }

    pub fn start(&mut self, device_idx: usize) -> Option<()> {
        self.selected_device_idx = Some(device_idx);
        if let Some(thr) = self.listening_thread.take() {
            thr.join().ok()?;
        }

        self.listening_thread = Some(self.init_listening_thread()?);
        Some(())
    }

    fn init_listening_thread(&self) -> Option<JoinHandle<()>> {
        let config = InputConfig::default();
        let mut input = self.build_selected_input(config)?;

        let ltc_config = LtcReaderConfig {
            frame_rate: self.frame_rate,
            max_speed: 2.0,
            min_amplitude: 1e-3,
            sample_rate: config.sample_rate.into(),
        };
        let mut tc_decoder = LtcReader::new(ltc_config);
        let sender = self.send.clone();

        Some(thread::spawn(move || {
            loop {
                let chunk = input.by_ref().take(256).collect::<Vec<f32>>();
                let res = tc_decoder.process_samples(&chunk);

                if let Ok(Some(time)) = res {
                    let blob = Ok((tc_decoder.sync_confidence(), time));
                    if sender.send(blob).is_err() {
                        break;
                    }
                } else if let Err(e) = res {
                    let _ = sender.send(Err(e));
                }
            }
        }))
    }

    fn build_selected_input(&self, config: InputConfig) -> Option<Microphone> {
        let device =
            Some(rodio::microphone::available_inputs().ok()?[self.selected_device_idx?].clone())?;
        let input = MicrophoneBuilder::new()
            .device(device)
            .ok()?
            .config(config)
            .ok()?
            .open_stream()
            .ok()?;
        Some(input)
    }

    pub fn available_devices(&self) -> &[String] {
        &self.available_devices
    }

    pub fn set_frame_rate(&mut self, frame_rate: FrameRate) {
        self.frame_rate = frame_rate;
    }

    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }

    pub fn selected_device_idx(&self) -> Option<usize> {
        self.selected_device_idx
    }
}

impl Default for TimecodeReader {
    fn default() -> Self {
        Self::new()
    }
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
