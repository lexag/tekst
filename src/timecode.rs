use ks_common_generic::smpte::ltc::{LtcReader, LtcReaderConfig};
use ks_common_generic::smpte::{FrameRate, Timecode, TimecodeError};
use ks_common_ui::component_interface::ConfigurationWidget;
use ks_common_ui::components::selector_list_value;
use rodio::microphone::{InputConfig, MicrophoneBuilder};
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
        self.available_devices.clear();
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

impl ConfigurationWidget for TimecodeReader {
    fn draw_configuration(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            // AUDIO DEVICES
            if let Some(selection) = ks_common_ui::components::selector_list_index(
                ui,
                &self.available_devices,
                self.selected_device_idx(),
                "Audio Device",
            ) {
                self.start(selection);
            }

            // FRAMERATES
            if let Some(selection) = selector_list_value(
                ui,
                &[
                    FrameRate::Fps23976,
                    FrameRate::Fps24,
                    FrameRate::Fps25,
                    FrameRate::Fps2997DF,
                    FrameRate::Fps2997NDF,
                    FrameRate::Fps30,
                ],
                &self.frame_rate(),
                "Frame Rate",
            ) {
                self.frame_rate = selection;
            }
        })
        .response
    }

    fn grid_contents(&mut self, ui: &mut egui::Ui) {
        unimplemented!()
    }
}
