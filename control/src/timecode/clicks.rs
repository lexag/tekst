use crate::{
    errorlog::log_if_error,
    timecode::{
        LTCReaderError, RenderableTimecodeHypothesis, TimecodeHypothesis, TimecodeReaderWidget,
    },
};
use egui::{Color32, NumExt, Widget};
use ks_common_generic::str::StaticString;
use ks_common_generic::{
    network::IpAddress,
    smpte::{
        FrameRate,
        ltc::{LtcReader, LtcReaderConfig, TimecodeReader},
    },
};
use ks_common_ui::{
    components::{self, Popup},
    material_icons, style,
    traits::{
        AutoInlineWidgetMenu, ConfigurationWidget, InlineWidget, InlineWidgetAutoEnum,
        InlineWidgetMenu, SubstitutedAutoInlineWidgetMenu,
    },
};
use local_ip_address::local_ip;
use std::{
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    str::FromStr,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct ClicksLTCReader {
    last_seen_timecode: TimecodeHypothesis,
    expected_frame_rate: FrameRate,

    pub listening_thread: Option<JoinHandle<()>>,
    request_thread_stop: Arc<Mutex<bool>>,
    recv: Receiver<Result<TimecodeHypothesis, LTCReaderError>>,
    send: Sender<Result<TimecodeHypothesis, LTCReaderError>>,

    address: SocketAddrV4,
}

impl TimecodeReader<LTCReaderError> for ClicksLTCReader {
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

impl ClicksLTCReader {
    pub fn new() -> Self {
        let (send, recv) = std::sync::mpsc::channel::<Result<TimecodeHypothesis, LTCReaderError>>();
        Self {
            last_seen_timecode: TimecodeHypothesis::default(),
            expected_frame_rate: FrameRate::Fps25,
            listening_thread: None,
            request_thread_stop: Arc::new(Mutex::new(false)),
            recv,
            send,
            address: SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 1), 0),
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
        let socket = UdpSocket::bind(SocketAddrV4::new(
            Ipv4Addr::from_str(&local_ip()?.to_string())?,
            0,
        ))?;

        let sender = self.send.clone();

        socket.set_nonblocking(true);

        socket.connect(self.address);

        let subscribe_message = ks_common_clicks::protocol::request::Request::Subscribe(
            ks_common_generic::network::SubscriberInfo {
                identifier: StaticString::new("tekst"),
                address: IpAddress::from_address_str(&socket.local_addr()?.to_string())
                    .ok_or(LTCReaderError::Unknown)?,
                // FIXME: this unknown error is ugly, it comes from the fact that ks_common
                // uses a proprietary IpAddress type, which should be switched out for the std
                // (core) version at some point
                message_kinds: ks_common_generic::typeflags::MessageType::TimecodeData,
                last_contact: 0,
            },
        );
        let request_stop = self.request_thread_stop.clone();

        let mut encoded = [0; 512];
        let out = postcard::to_slice(&subscribe_message, &mut encoded)?;
        socket.send(&out);

        let mut buf = [0u8; 256];
        Ok(thread::spawn(move || {
            loop {
                if request_stop.try_lock().is_ok_and(|b| *b) {
                    break;
                }

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

                if sender.send(Ok((tc.ltc, 1.0))).is_err() {
                    println!("tc send error");
                    break;
                };
            }
        }))
    }
}

impl SubstitutedAutoInlineWidgetMenu<RenderableTimecodeHypothesis> for ClicksLTCReader {
    fn substitute(&self) -> RenderableTimecodeHypothesis {
        self.last_seen_timecode.into()
    }
}

impl ConfigurationWidget for ClicksLTCReader {
    fn grid_contents(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.expected_frame_rate
                    .autoenum_inline_widget_menu(ui, "Frame rate");

                if components::Button::new("Start/stop network connection")
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

            self.address.auto_inline_widget_menu(ui, "Core ip address")
        });
    }
}
