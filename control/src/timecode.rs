use ks_common_generic::network::IpAddress;
use ks_common_generic::smpte::ltc::{LtcReader, LtcReaderConfig};
use ks_common_generic::smpte::{FrameRate, Timecode, TimecodeError};
use ks_common_generic::str::StaticString;
use ks_common_ui::component_interface::{ConfigurationWidget, InlineWidget, InlineWidgetMenu};
use ks_common_ui::components::selector_list_value;
use local_ip_address::local_ip;
use rodio::microphone::{InputConfig, MicrophoneBuilder};
use rodio::{DeviceTrait, microphone::Microphone};
use std::fmt::Display;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::{
    ops::Sub,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

type TcBlob = Result<(f32, Timecode), TimecodeError>;

#[derive(Clone, Copy, PartialEq)]
enum TimecodeDevice {
    LTCDevice(Option<usize>),
    ClicksTCDevice(SocketAddrV4),
}

impl Display for TimecodeDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimecodeDevice::LTCDevice(_) => write!(f, "LTC Device"),
            TimecodeDevice::ClicksTCDevice(_) => write!(f, "ClicKS TC Device"),
        }
    }
}

pub struct TimecodeReader {
    listening_thread: Option<JoinHandle<()>>,
    confidence: f32,
    last_timecode: Option<Timecode>,
    frame_rate: FrameRate,
    recv: Receiver<TcBlob>,
    send: Sender<TcBlob>,
    available_devices: Vec<String>,
    selected_device: TimecodeDevice,
}

impl TimecodeReader {
    pub fn new() -> Self {
        let (snd, rec) = mpsc::channel();
        let mut a = Self {
            available_devices: vec![],
            listening_thread: None,
            selected_device: TimecodeDevice::LTCDevice(None),
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

    pub fn start(&mut self) -> Option<()> {
        match self.selected_device {
            TimecodeDevice::LTCDevice(idx) => self.start_ltc_thread(idx?),
            TimecodeDevice::ClicksTCDevice(addr) => self.start_clickstc_thread(addr),
        }
    }

    fn start_ltc_thread(&mut self, device_idx: usize) -> Option<()> {
        self.selected_device = TimecodeDevice::LTCDevice(Some(device_idx));
        self.stop_previous_listening_thread()?;

        self.listening_thread = Some(self.init_listening_thread()?);
        Some(())
    }

    fn start_clickstc_thread(&mut self, addr: SocketAddrV4) -> Option<()> {
        self.selected_device = TimecodeDevice::ClicksTCDevice(addr);
        self.stop_previous_listening_thread()?;
        self.listening_thread = Some(self.init_clicks_listening_thread()?);

        Some(())
    }
    fn stop_previous_listening_thread(&mut self) -> Option<()> {
        Some(if let Some(thr) = self.listening_thread.take() {
            thr.join().ok()?;
        })
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

    fn init_clicks_listening_thread(&self) -> Option<JoinHandle<()>> {
        let socket = UdpSocket::bind(SocketAddrV4::new(
            Ipv4Addr::from_str(&local_ip().unwrap().to_string()).unwrap(),
            0,
        ))
        .ok()?;

        let sender = self.send.clone();

        let TimecodeDevice::ClicksTCDevice(addr) = self.selected_device else {
            return None;
        };

        socket.connect(addr);

        let subscribe_message = ks_common_clicks::protocol::request::Request::Subscribe(
            ks_common_generic::network::SubscriberInfo {
                identifier: StaticString::new("tekst"),
                address: IpAddress::from_address_str(&socket.local_addr().ok()?.to_string())?,
                message_kinds: ks_common_generic::typeflags::MessageType::TimecodeData,
                last_contact: 0,
            },
        );

        let mut encoded = [0; 512];
        let out = postcard::to_slice(&subscribe_message, &mut encoded).ok()?;
        socket.send(&out);

        let mut buf = [0u8; 256];
        Some(thread::spawn(move || {
            loop {
                let Ok(size) = socket.recv(&mut buf) else {
                    continue;
                };

                if size == 0 {
                    continue;
                }

                let Ok(msg) = postcard::from_bytes::<
                    ks_common_clicks::protocol::message::SmallMessage,
                >(&buf[1..size]) else {
                    continue;
                };

                let ks_common_clicks::protocol::message::SmallMessage::TimecodeData(tc) = msg
                else {
                    continue;
                };

                if sender.send(Ok((100.0, tc.ltc))).is_err() {
                    println!("tc send error");
                    break;
                };
            }
        }))
    }

    fn build_selected_input(&self, config: InputConfig) -> Option<Microphone> {
        let TimecodeDevice::LTCDevice(idx) = self.selected_device else {
            return None;
        };
        let device = Some(rodio::microphone::available_inputs().ok()?[idx?].clone())?;
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

    pub fn selected_device(&self) -> TimecodeDevice {
        self.selected_device
    }

    pub fn selected_device_type(&self) -> Option<usize> {
        match self.selected_device {
            TimecodeDevice::LTCDevice(_) => Some(0),
            TimecodeDevice::ClicksTCDevice(_) => Some(1),
        }
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
            const OPTIONS: &[TimecodeDevice] = &[
                TimecodeDevice::LTCDevice(None),
                TimecodeDevice::ClicksTCDevice(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 0), 0)),
            ];
            if let Some(tc_type_selection) = ks_common_ui::components::selector_list_index(
                ui,
                OPTIONS,
                self.selected_device_type(),
                "Timecode Device",
            ) {
                self.selected_device = OPTIONS[tc_type_selection];
            }

            match &mut self.selected_device {
                TimecodeDevice::LTCDevice(device_idx) => {
                    // AUDIO DEVICES
                    if let Some(selection) = ks_common_ui::components::selector_list_index(
                        ui,
                        &self.available_devices,
                        *device_idx,
                        "Audio Device",
                    ) {
                        *device_idx = Some(selection);
                    }
                }

                TimecodeDevice::ClicksTCDevice(addr) => {
                    ui.vertical(|ui| {
                        addr.inline_widget(ui);
                        addr.draw_configuration(ui);
                    });
                }
                _ => {}
            }
            if ui.button("START").clicked() {
                self.start();
            }
        })
        .response
    }

    fn grid_contents(&mut self, ui: &mut egui::Ui) {
        unimplemented!()
    }
}
