// /// {{ example.rs }}

use gst::{glib::SendValue, message::Application, prelude::*, *};
use gst_app::{AppSink, AppSinkCallbacks};
use gst_video::*;
use std::{
    collections::HashMap,
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock, RwLock,
    },
    time::{Duration, Instant},
};
use tgui::{
    items::{self, Visibility},
    Vi,
};
use url::Url;

pub static DATA: OnceLock<Data> = OnceLock::new();
pub static FLAG_EXIT: AtomicBool = AtomicBool::new(false);

pub fn loop_bus(
    pipeline: Pipeline,
    args: Args,
    view_map: Arc<RwLock<HashMap<(i32, i32), &'static str>>>,
) -> tgui::Res<()> {
    let mut position = 0;
    dbg!("loop_bus");

    pipeline.set_state(State::Paused).unwrap();

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    let mut click_gesture = ClickGesture::default();
    'l: for msg in bus.iter_timed(ClockTime::NONE) {
        // gst
        match msg.view() {
            // with `sync=false`
            //MessageView::AsyncDone(..) => {
            //}

            // with `sync=true`
            MessageView::DurationChanged(..) => {
                while pipeline
                    .seek_simple(SeekFlags::FLUSH, position * ClockTime::SECOND)
                    .is_err()
                {}
                pipeline.set_state(State::Playing).unwrap();

                position += 1;
            }

            MessageView::Application(v) => {

                //let Ok(msg) = v.structure().unwrap().get("status") else {
                //    continue;
                //};
                //
                //dbg!(&msg);
                //match msg {
                //    "paused" => {
                //        pipeline.set_state(State::Paused).unwrap();
                //
                //        break;
                //    }
                //    _ => {}
                //}
            }

            MessageView::StreamStart(..) => {
                println!("StreamStart");
            }

            // The End-of-stream message is posted when the stream is done, which in our case
            // happens immediately after creating the thumbnail because we return
            // FlowError::Eos then.
            MessageView::Eos(..) => {
                // Exit
                if args.loop_nums == 1 {
                    println!("Got Eos message, exit");
                    pipeline.set_state(State::Null).unwrap();

                    break 'l;
                }
                // Infinite loop
                else if args.loop_nums == 0 {
                    position = 0;

                    while pipeline
                        .seek_simple(SeekFlags::FLUSH, ClockTime::ZERO)
                        .is_err()
                    {}
                    pipeline.set_state(State::Playing).unwrap();
                } else {
                    // for in 0...max
                    todo!()
                }
            }

            MessageView::Error(err) => {
                let _ = pipeline.set_state(State::Null);
                eprintln!("{err:#?}");
                break 'l;
            }

            _ => {}
        }

        // window
        let Some(Data { act, pg }) = DATA.get() else {
            continue 'l;
        };
        let Some(e) = act.get_event()? else {
            continue 'l;
        };

        match e {
            // TODO:
            //items::event::Event::Touch(..) => 's: {}
            items::event::Event::Click(items::ClickEvent { v, .. }) => 's: {
                let view_map = view_map.read().unwrap();
                let items::View { aid, id } = v.unwrap();
                let Some(name) = view_map.get(&(aid, id)) else {
                    break 's;
                };

                if name == &"pg" {
                    dbg!(&pipeline.current_clock_time());
                    break 's;
                }

                if click_gesture.is_one() {
                    dbg!("Maybe Double Click");

                    let now = click_gesture.dur();

                    // if too fast
                    if now < Duration::from_millis(100) {
                        break 's;
                    }

                    // if timeout
                    if now > Duration::from_millis(200) {
                        //dbg!("timeout");

                        click_gesture.reset();

                        break 's;
                    }

                    //dbg!("Couble Click");

                    let seek_step = 5;
                    match *name {
                        "video" => match pipeline.current_state() {
                            State::Playing => {
                                // TODO: pgbar
                                pg.vi_visible(Visibility::Visible)?;
                                let _ = pipeline.set_state(State::Paused);
                            }
                            State::Paused => {
                                pg.vi_visible(Visibility::Hidden)?;
                                let _ = pipeline.set_state(State::Playing);
                            }
                            _ => {}
                        },
                        "libtn" => 's: {
                            if position < seek_step {
                                break 's;
                            }
                            position -= seek_step;

                            while pipeline
                                .seek_simple(
                                    SeekFlags::FLUSH | SeekFlags::KEY_UNIT,
                                    position * ClockTime::SECOND,
                                )
                                .is_err()
                            {}
                        }
                        "ribtn" => 's: {
                            dbg!("seek next");

                            position += seek_step;
                            while pipeline
                                .seek_simple(
                                    SeekFlags::FLUSH | SeekFlags::KEY_UNIT,
                                    position * ClockTime::SECOND,
                                )
                                .is_err()
                            {}
                            pipeline.set_state(State::Playing).unwrap();
                        }
                        _ => {}
                    }

                    click_gesture.reset();
                } else {
                    //dbg!("Single Click");

                    click_gesture.timing();

                    break 's;
                }
            }

            items::event::Event::Back(items::BackButtonEvent { .. }) => {
                dbg!("Back");

                // exit main()
                FLAG_EXIT.store(true, Ordering::Relaxed);

                // exit loop-bus()
                break 'l;
            }

            items::event::Event::Start(items::StartEvent { .. }) => {
                dbg!("start");

                act.event_intercept_volume(true, true)?;
                act.event_intercept_back()?;
            }

            _ => {}
        }
    }

    while pipeline.set_state(State::Null).is_err() {}

    Ok(())
}

pub fn loop_callback(res: Arc<RwLock<WrapBuffer>>, uri: String) -> NewPipe {
    gst::init().unwrap();

    let uri = {
        if let Some(name) = uri.strip_prefix("./") {
            let cur = env::current_dir().unwrap().display().to_string();

            format!("file://{cur}/{name}")
        } else if let Some(name) = uri.strip_prefix('~') {
            let cur = env::home_dir().unwrap().display().to_string();

            format!("file://{cur}/{name}")
        } else {
            // maybe { http, udp, ... }
            uri.to_string()
        }
    };

    let uri = Url::parse(&uri).unwrap().to_string();
    dbg!(&uri);

    //de. ! queue ! audioconvert ! audioresample ! appsink name=aud
    let pipeline = format!(
        "
uridecodebin uri={uri} name=de
    de. ! queue ! audioconvert ! audioresample ! autoaudiosink name=aud
    de. ! queue ! videoconvert ! appsink name=vid
"
    );
    //de. ! qtdemux ! fakesink name=vid_info
    let pipeline = parse::launch(&pipeline).unwrap();
    let pipeline = pipeline
        .downcast::<Pipeline>()
        .expect("Expected a Pipeline");

    // Get access to the appsink element.
    let vid_sink = pipeline
        .by_name("vid")
        .expect("Sink element not found")
        .downcast::<AppSink>()
        .expect("Sink element is expected to be an appsink!");
    //let vid_pad = vid_sink.static_pad("vid").unwrap();
    //    let vid_total_time = vid_sink
    //        .query_duration::<gst::ClockTime>()
    //        .unwrap()
    //        .seconds();
    //    dbg!(vid_total_time);

    //    def probe_callback(hlssink_pad,info):
    //    info_event = info.get_event()
    //    info_structure = info_event.get_structure()
    //    do_something_with_this_info
    //    return Gst.PadProbeReturn.PASS
    let aud_sink = pipeline.by_name("aud").expect("Sink element not found");
    //    let vid_info = pipeline
    //        .by_name("vid_info")
    //        .expect("Sink element not found");
    //dbg!(&vid_info.metadata("description"));
    //dbg!(&vid_info.metadata("klass"));
    //dbg!(&vid_info.metadata("name"));

    //    let aud_sink = pipeline
    //        .by_name("aud")
    //        .expect("Sink element not found")
    //        .downcast::<AppSink>()
    //        .expect("Sink element is expected to be an appsink!");

    // Don't synchronize on the clock, we only want a snapshot asap.
    //appsink.set_property("sync", false);
    vid_sink.set_property("sync", true);
    aud_sink.set_property("sync", true);

    // Tell the appsink what format we want.
    // This can be set after linking the two objects, because format negotiation between
    // both elements will happen during pre-rolling of the pipeline.
    let vid_caps = VideoCapsBuilder::new().format(VideoFormat::Rgba).build();
    vid_sink.set_caps(Some(&vid_caps));

    //    let size = get_buffer_size(&vid_caps);
    //    fn get_buffer_size(caps: &Caps) -> Option<(IntRange<i32>, IntRange<i32>)> {
    //        let Some(caps_struct) = caps.structure(0) else {
    //            return None;
    //        };
    //
    //        dbg!(&caps_struct);
    //
    //        let width = caps_struct.get("width").unwrap();
    //        let height = caps_struct.get("height").unwrap();
    //
    //        Some((width, height))
    //    }
    //    let w: glib::Value = IntRange::to_value(&size.unwrap().0);
    //
    //    dbg!(&size, &w);

    //let aud_caps = AudioCapsBuilder::new().format(AudioFormat::U32le).build();
    //aud_sink.set_caps(Some(&aud_caps));

    // Add a handler to the "new-sample" signal.
    let vid_f = move |vid_sink: &AppSink| 's1: {
        //dbg!("video");

        // Pull the sample in question out of the appsink's buffer.
        let sample = vid_sink.pull_sample().map_err(|_| FlowError::Eos)?;
        let buffer = sample.buffer().ok_or_else(|| {
            element_error!(
                vid_sink,
                ResourceError::Failed,
                ("Failed to get buffer from vid_sink")
            );

            FlowError::Error
        })?;

        // Make sure that we only get a single buffer
        //if got_snapshot {
        //    return Err(FlowError::Eos);
        //}
        //got_snapshot = true;

        let caps = sample.caps().expect("Sample without caps");
        let info = VideoInfo::from_caps(caps).expect("Failed to parse caps");

        //let timestamp = buffer.pts().unwrap();
        //dbg!(timestamp);

        let frame = VideoFrameRef::from_buffer_ref_readable(buffer, &info).map_err(|_| {
            element_error!(
                vid_sink,
                ResourceError::Failed,
                ("Failed to map buffer readable")
            );

            FlowError::Error
        })?;

        // TODO: get I-P-B Frame

        //println!("Have video frame");

        let width = info.width();
        let height = info.height();
        //let format = info.format();

        let plane_data = frame.plane_data(0).unwrap();

        's2: {
            let mut res = res.write().unwrap();

            //dbg!(&res.msg);
            //match res.msg {
            //    AppMsg::ToggleState => {
            //        if res.status == AppState::Played {
            //            res.status = AppState::Paused;
            //
            //            let mut s = Structure::new_empty("status");
            //            s.set_value("paused", SendValue::from(""));
            //
            //            let msg = Application::new(s);
            //            vid_sink.post_message(msg).unwrap();
            //
            //            break 's1 Ok(FlowSuccess::Ok);
            //        } else if res.status == AppState::Paused {
            //            res.status = AppState::Played;
            //        }
            //
            //        res.msg = AppMsg::Unknown;
            //    }
            //    _ => {}
            //}

            if res.is_synced {
                break 's2;
            }

            // init
            if res.inner.width == 0 {
                res.size = (width, height);
                cold(&mut res.inner, width, height);

                #[cold]
                #[inline]
                fn cold(res: &mut tgui::Buffer, width: u32, height: u32) {
                    *res = tgui::Buffer::zero(width, height).unwrap();
                }
            }

            res.inner.data.copy_from_slice(plane_data);
            res.is_synced = true;
            res.cur_time = vid_sink
                .query_position::<gst::ClockTime>()
                .unwrap()
                .seconds();
            res.total_time = vid_sink
                .query_duration::<gst::ClockTime>()
                .unwrap()
                .seconds();

            //dbg!(&res.width);
        }

        Ok(FlowSuccess::Ok)
    };

    //    let aud_f = move |aud_sink: &AppSink| {
    //        //dbg!("decode audio");
    //
    //        // Pull the sample in question out of the appsink's buffer.
    //        let sample = aud_sink.pull_sample().map_err(|_| FlowError::Eos)?;
    //        let buffer = sample.buffer().ok_or_else(|| {
    //            element_error!(
    //                aud_sink,
    //                ResourceError::Failed,
    //                ("Failed to get buffer from aud_sink")
    //            );
    //
    //            FlowError::Error
    //        })?;
    //
    //        let caps = sample.caps().expect("Sample without caps");
    //        let info = AudioInfo::from_caps(caps).expect("Failed to parse caps");
    //
    //        let map = buffer.map_readable().unwrap();
    //        let pcm_data = map.as_slice();
    //        //dbg!(&pcm_data.len());
    //
    //        //Err(FlowError::Eos)
    //        Ok(FlowSuccess::Ok)
    //    };

    // Getting data out of the appsink is done by setting callbacks on it.
    // The appsink will then call those handlers, as soon as data is available.
    let vid_callbacks = AppSinkCallbacks::builder().new_sample(vid_f).build();
    vid_sink.set_callbacks(vid_callbacks);

    //let aud_callbacks = AppSinkCallbacks::builder().new_sample(aud_f).build();
    //aud_sink.set_callbacks(aud_callbacks);

    NewPipe {
        pipeline,
        width: 0,
        height: 0,
    }
}

pub struct NewPipe {
    pub pipeline: Pipeline,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct WrapBuffer {
    pub size: (u32, u32),
    pub inner: tgui::Buffer,
    pub is_synced: bool,
    pub msg: AppMsg,
    pub status: AppState,

    pub cur_time: u64,
    pub total_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Played,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMsg {
    ToggleState,
    ShowTimestamp { format: TimestampFormat },

    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TimestampFormat {
    Pts,

    // Human readable
    Human,
}

#[derive(Default)]
struct ClickGesture {
    ty: ClickTy,
    timer: Option<Instant>,
}

#[derive(Default, PartialEq, Eq)]
enum ClickTy {
    #[default]
    Zero,
    One,
    Two,
}

#[derive(Debug, Default)]
pub struct Args {
    uri: String,
    loop_nums: usize,
}

#[derive(Debug, Clone)]
pub struct Data {
    act: tgui::Activity,
    pg: tgui::LinearLayout,
}

impl Data {
    pub fn new(act: tgui::Activity, pg: tgui::LinearLayout) -> Self {
        Self { act, pg }
    }
}

impl Args {
    pub fn new() -> Self {
        let mut args = std::env::args().collect::<Vec<_>>();

        let mut iter = args.iter_mut();
        let mut res = Self::default();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--uri" => {
                    res.uri = iter.next().unwrap().to_string();
                }
                "--loop-nums" => {
                    res.loop_nums = iter.next().unwrap().parse().unwrap();
                }
                _ => {}
            }
        }

        res
    }

    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }
}

impl WrapBuffer {
    pub fn new(inner: tgui::Buffer, size: (u32, u32)) -> Self {
        Self {
            size,
            inner,
            is_synced: false,
            msg: AppMsg::Unknown,
            status: AppState::Played,
            cur_time: 0,
            total_time: 0,
        }
    }

    pub fn fmt_cur_time(&self) -> String {
        fmt_time(self.cur_time)
    }

    pub fn fmt_total_time(&self) -> String {
        fmt_time(self.total_time)
    }
}

fn fmt_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl ClickGesture {
    pub fn is_zero(&self) -> bool {
        self.ty == ClickTy::Zero
    }

    pub fn is_one(&self) -> bool {
        self.ty == ClickTy::One
    }

    pub fn is_two(&self) -> bool {
        self.ty == ClickTy::Two
    }

    pub fn timing(&mut self) {
        self.ty = ClickTy::One;
        self.timer = Some(Instant::now());
    }

    pub fn dur(&self) -> Duration {
        self.timer.unwrap().elapsed()
    }

    pub fn reset(&mut self) {
        *self = Self::default()
    }
}

// REFS: https://developer.android.com/reference/android/media/MediaCodec.html
