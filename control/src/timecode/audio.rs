use crate::{
    errorlog::log_if_error,
    timecode::{
        LTCReaderError, RenderableTimecodeHypothesis, TimecodeHypothesis, TimecodeReaderWidget,
    },
};
use egui::{Color32, NumExt, Widget};
use ks_common_generic::smpte::{
    FrameRate,
    ltc::{LtcReader, LtcReaderConfig, TimecodeReader},
};
use ks_common_generic::str::StaticString;
use ks_common_ui::{
    components::{self, Popup},
    material_icons, style,
    traits::{
        ConfigurationWidget, InlineWidget, InlineWidgetAutoEnum, InlineWidgetMenu,
        SubstitutedAutoInlineWidgetMenu,
    },
};
use rodio::DeviceTrait;
use rodio::microphone::{InputConfig, Microphone, MicrophoneBuilder};
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct AudioLTCReader {
    last_seen_timecode: TimecodeHypothesis,
    expected_frame_rate: FrameRate,
    listening_thread: Option<JoinHandle<()>>,
    request_thread_stop: Arc<Mutex<bool>>,
    recv: Receiver<Result<TimecodeHypothesis, LTCReaderError>>,
    send: Sender<Result<TimecodeHypothesis, LTCReaderError>>,
    available_devices: Vec<String>,
    selected_device: Option<usize>,
}

impl TimecodeReader<LTCReaderError> for AudioLTCReader {
    fn read_timecode(
        &mut self,
    ) -> Result<Option<ks_common_generic::smpte::Timecode>, LTCReaderError> {
        let (tc, conf) = self.read_timecode_confidence()?;

        Ok(if conf < 0.01 { None } else { Some(tc) })
    }

    fn frame_rate(&self) -> ks_common_generic::smpte::FrameRate {
        todo!()
    }

    fn is_synchronized(&self) -> bool {
        todo!()
    }

    fn read_timecode_confidence(
        &mut self,
    ) -> Result<(ks_common_generic::smpte::Timecode, f32), LTCReaderError> {
        self.last_seen_timecode = self
            .ask_listening_thread()
            .map(|o| o.unwrap_or(self.last_seen_timecode))?;
        Ok(self.last_seen_timecode)
    }
}

impl AudioLTCReader {
    pub fn new() -> Self {
        let (send, recv) = std::sync::mpsc::channel::<Result<TimecodeHypothesis, LTCReaderError>>();
        Self {
            last_seen_timecode: TimecodeHypothesis::default(),
            expected_frame_rate: FrameRate::Fps25,
            listening_thread: None,
            request_thread_stop: Arc::new(Mutex::new(false)),
            recv,
            send,
            available_devices: vec![],
            selected_device: None,
        }
    }

    pub fn ask_listening_thread(&mut self) -> Result<Option<TimecodeHypothesis>, LTCReaderError> {
        self.last_seen_timecode.1 -= 0.01;
        self.last_seen_timecode.1 = self.last_seen_timecode.1.at_least(0.0);
        let mut time = None;
        while let Ok(res) = self.recv.try_recv() {
            match res {
                Ok(hyp) => {
                    time = Some(hyp);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(time)
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

    pub fn start(&mut self) -> Result<(), LTCReaderError> {
        let _thread_was_running = self.stop_previous_listening_thread()?;
        self.listening_thread = Some(self.init_listening_thread()?);
        Ok(())
    }

    fn stop_previous_listening_thread(&mut self) -> Result<bool, LTCReaderError> {
        if let Some(thr) = self.listening_thread.take() {
            // FIXME: this error mapping is not too nice.
            *self
                .request_thread_stop
                .lock()
                .map_err(|_| LTCReaderError::RodioInternal)? = true;

            // we wait for the listening thread to read the request and shut down nicely
            thread::sleep(Duration::from_secs(1));
            Ok(thr.join().map(|()| true)?)
        } else {
            Ok(false)
        }
    }

    fn init_listening_thread(&self) -> Result<JoinHandle<()>, LTCReaderError> {
        let config = InputConfig::default();
        let mut input = self.build_selected_input(config)?;

        let request_stop = self.request_thread_stop.clone();

        let ltc_config = LtcReaderConfig {
            frame_rate: self.expected_frame_rate,
            max_speed: 2.0,
            min_amplitude: 1e-3,
            sample_rate: config.sample_rate.into(),
        };
        let mut tc_decoder = LtcReader::new(ltc_config);
        let sender = self.send.clone();

        Ok(thread::spawn(move || {
            loop {
                let chunk = input.by_ref().take(256).collect::<Vec<f32>>();
                match tc_decoder.process_samples(&chunk) {
                    Ok(_) => {
                        let res = tc_decoder.read_timecode_confidence();
                        sender.send(res.map_err(|e| LTCReaderError::Timecode(e)));
                    }
                    Err(e) => {
                        sender.send(Err(e.into()));
                    }
                }
                if request_stop.try_lock().is_ok_and(|b| *b) {
                    break;
                }
            }
        }))
    }

    fn build_selected_input(&self, config: InputConfig) -> Result<Microphone, LTCReaderError> {
        let device = rodio::microphone::available_inputs()?
            [self.selected_device.ok_or(LTCReaderError::NoSelection)?]
        .clone();
        let input = MicrophoneBuilder::new()
            .device(device)
            // this map_err is unfortunate but
            // rodio does not export
            // microphone::builder::Error which
            // is what this returns
            .map_err(|_| LTCReaderError::RodioInternal)?
            .config(config)
            .map_err(|_| LTCReaderError::RodioInternal)?
            .open_stream()?;
        Ok(input)
    }
}

impl SubstitutedAutoInlineWidgetMenu<RenderableTimecodeHypothesis> for AudioLTCReader {
    fn substitute(&self) -> RenderableTimecodeHypothesis {
        self.last_seen_timecode.into()
    }
}

impl ConfigurationWidget for AudioLTCReader {
    fn grid_contents(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.expected_frame_rate
                    .autoenum_inline_widget_menu(ui, "Frame rate");

                if components::Button::new("Refresh audio devices")
                    .icon(material_icons::Icon::Refresh)
                    .ui(ui)
                    .clicked()
                {
                    self.reload_available_devices();
                }

                if components::Button::new("Start/stop audio device")
                    .icon(material_icons::Icon::PowerSettingsNew)
                    .indicator(
                        self.listening_thread
                            .is_some()
                            .then_some(style::ACTIVE_COLOR),
                    )
                    .ui(ui)
                    .clicked()
                {
                    if self.listening_thread.is_some() {
                        log_if_error(ui, self.stop_previous_listening_thread());
                    } else {
                        log_if_error(ui, self.start());
                    }
                };
            });

            StaticString::<32>::new(
                self.selected_device
                    .and_then(|i| self.available_devices.get(i))
                    .map_or("No audio device selected", |d| d),
            )
            .inline_widget_menu(ui, "Audio device", |ui| {
                if let Some(selection) = components::selector_list_index(
                    ui,
                    &self.available_devices,
                    self.selected_device,
                ) {
                    self.selected_device = Some(selection);
                }
            });
        });
    }
}
