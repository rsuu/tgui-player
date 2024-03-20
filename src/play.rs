// /// {{ example.rs }}

use gst::{glib::SendValue, message::Application, prelude::*, *};
use gst_app::{AppSink, AppSinkCallbacks};
use gst_video::*;
use std::{
    env,
    sync::{Arc, OnceLock, RwLock},
    thread::sleep_ms,
    time::{Duration, Instant, SystemTime},
};
use tgui::items;
use url::Url;

pub static DATA: OnceLock<Data> = OnceLock::new();

pub fn main(pipeline: Pipeline, args: Args) -> tgui::Res<()> {
    let mut position = 0;

    pipeline.set_state(State::Paused).unwrap();

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    let mut timeout = SystemTime::now();
    let mut touch_count = 0;
    let mut flag_double_click = false;
    'l: for msg in bus.iter_timed(ClockTime::NONE) {
        match msg.view() {
            // with `sync=false`
            //MessageView::AsyncDone(..) => {
            //}

            // with `sync=true`
            MessageView::DurationChanged(..) => {
                let _ = pipeline.seek_simple(SeekFlags::FLUSH, position * ClockTime::SECOND);
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
                    let _ = pipeline.seek_simple(SeekFlags::FLUSH, ClockTime::ZERO);
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

        let Some(Data { app, act }) = DATA.get() else {
            continue 'l;
        };
        let Some(e) = app.get_event()? else {
            continue 'l;
        };

        match e {
            // TODO:
            //items::event::Event::Touch(..) => 's: {}
            items::event::Event::Click(items::ClickEvent { v, .. }) => 's: {
                // double click
                if flag_double_click {
                    // if !timeout
                    let cur = SystemTime::now();

                    if cur < timeout + Duration::from_millis(100) {
                        break 's;
                    }

                    if cur > timeout + Duration::from_millis(250) {
                        println!("timeout");

                        flag_double_click = false;
                        timeout = cur;

                        break 's;
                    }

                    dbg!("Double Click");

                    match pipeline.current_state() {
                        State::Playing => {
                            let _ = pipeline.set_state(State::Paused);
                        }
                        State::Paused => {
                            let _ = pipeline.set_state(State::Playing);
                        }
                        _ => {}
                    }

                    flag_double_click = false;
                    timeout = cur;

                // single click
                } else {
                    dbg!("Single Click");

                    flag_double_click = true;
                    timeout = SystemTime::now();

                    break 's;
                }
            }

            items::event::Event::Back(items::BackButtonEvent { .. }) => {
                dbg!("Back");

                let _ = pipeline.set_state(State::Null);
                bus.remove_signal_watch();

                app.close();
                return Ok(());
            }

            items::event::Event::Start(items::StartEvent { .. }) => {
                dbg!("start");

                app.event_intercept_volume(act, true, true)?;
                app.event_intercept_back(act)?;
            }

            _ => {}
        }
    }

    pipeline.set_state(State::Null).unwrap();

    Ok(())
}

pub fn create_pipeline(res: Arc<RwLock<WrapBuffer>>, uri: String) -> NewPipe {
    init().unwrap();

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

    let arc_start_time = Arc::new(SystemTime::now());
    let start_time = arc_start_time.clone();
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
            match res.msg {
                AppMsg::ToggleState => {
                    if res.status == AppState::Played {
                        res.status = AppState::Paused;

                        let mut s = Structure::new_empty("status");
                        s.set_value("paused", SendValue::from(""));

                        let msg = Application::new(s);
                        vid_sink.post_message(msg).unwrap();

                        break 's1 Ok(FlowSuccess::Ok);
                    } else if res.status == AppState::Paused {
                        res.status = AppState::Played;
                    }

                    res.msg = AppMsg::Unknown;
                }
                _ => {}
            }

            // init vec
            if res.inner.width == 0 {
                res.size = (width, height);
                cold(&mut res.inner, width, height);

                #[cold]
                fn cold(res: &mut tgui::Buffer, width: u32, height: u32) {
                    *res = tgui::Buffer::zero(width, height).unwrap();
                }
            }

            if !res.is_synced {
                res.inner.data.copy_from_slice(plane_data);
                res.is_synced = true;
            }

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

#[derive(Debug, Default)]
pub struct Args {
    uri: String,
    loop_nums: usize,
}

#[derive(Debug, Clone)]
pub struct Data {
    app: tgui::Tgui,
    act: tgui::Activity,
}

impl Data {
    pub fn new(app: tgui::Tgui, act: tgui::Activity) -> Self {
        Self { app, act }
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
        }
    }
}
