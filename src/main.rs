// TODO:
//   [ ] gst: metadata
//       [ ] width and height
//       [ ] duration time
//   [ ] tgup: notification
//   [ ] tgup: fullscreen
//   [ ] crate: Vec<u8> to Vec<u32>
//   [ ] crate: audio
//       [ ] pipewire
//   [ ] crate: editor
//       [ ] play/pause
//       [ ] seek
//       [ ] pgbar
//       [ ] thumb
//       [ ] trim
//       [ ] cut
//   [ ] crate: gui
//       [ ] mouse
//       [ ] button
//   [ ] crate: music player
//       [ ] widget
//       [ ] notification

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread::{self, sleep},
    time::Duration,
};
use tgui::{items::Visibility, View, ViewSet, *};
use tgui_player::{
    play::{self, Args, Data, WrapBuffer},
    //rect, PhyRect, Rect,
};

fn main() -> Res<()> {
    // ============== init ==============
    let arc_view_map = Arc::new(RwLock::new(HashMap::<(i32, i32), &'static str>::new()));

    // ============== Gstreamer ==============
    let args = Args::new();
    let arc_buffer = Arc::new(RwLock::new(WrapBuffer::new(Buffer::zero(0, 0)?, (0, 0))));
    dbg!(&args);

    let uri = args.uri().to_string();
    let buffer = arc_buffer.clone();
    let view_map = arc_view_map.clone();
    thread::spawn(|| {
        let pipe = play::loop_callback(buffer, uri);
        play::loop_bus(pipe.pipeline, args, view_map).unwrap();
    });

    // ============== Window ==============
    while arc_buffer.read().unwrap().inner.width == 0 {
        sleep(Duration::from_millis(10));
    }

    // Layout:
    //
    // <FrameLayout>
    //   <LinearLayout>
    //     <Image --center />
    //   </LinearLayout>
    //
    //   <LinearLayout>
    //     <Button --left />
    //     <Space --center />
    //     <Button --right />
    //   </LinearLayout>
    //
    //   <LinearLayout>
    //     <ProgressBar />
    //   </LinearLayout>
    // </FrameLayout>

    let task = Task::new()?.conn()?;
    let act = task.new_activity(-1)?;
    act.config_keep_screen_on()?;

    let tmp = arc_view_map.clone();
    let mut view_map = tmp.write().unwrap();

    dbg!(&act);
    // FrameLayout
    let layout_frame = FrameLayout::new(&act)
        .set_data(act.gen_create().unwrap().set_parent(-1))
        .conn()?;

    // LinearLayout - Image
    let layout_linear_img = act.new_layout_linear(&layout_frame, true)?;
    let alliv = act.gen_view(&layout_linear_img).unwrap();

    // ImageView
    let tmp = arc_buffer.read().unwrap().clone();
    //let (width, height) = tmp.size;
    let mut buffer_res = act.new_buffer(&tmp.inner)?;
    let data = act
        .gen_create()
        .unwrap()
        .set_parent(layout_linear_img.id()?);
    let img = Img::new(&act).set_data(data).conn()?;
    buffer_res.set(&act, &img)?;
    img.vi_clickable(true)?;
    img.vi_click_event(true)?;
    view_map.insert((img.act().aid()?, img.id()?), "video");
    //act.vi_touch_event(aiv.clone(), true)?;

    // LinearLayout - Button
    let layout_linear = act.new_layout_linear(&layout_frame, true)?;

    // TODO: replace Img with Button in the future
    // Left Button(ImageView)
    let data = act.gen_create().unwrap().set_parent(layout_linear.id()?);
    let libtn = Img::new(&act).set_data(data).conn()?;
    libtn.vi_click_event(true)?;
    act.set_layout_linear(&libtn, 0.25, 0)?;
    view_map.insert((libtn.act().aid()?, libtn.id()?), "libtn");

    // Space
    let space_view = act.new_space(&layout_linear)?;
    act.set_layout_linear(&space_view, 1.0, 0)?;

    // Right Button(ImageView)
    let data = act.gen_create().unwrap().set_parent(layout_linear.id()?);
    let ribtn = Img::new(&act).set_data(data).conn()?;
    ribtn.vi_click_event(true)?;
    act.set_layout_linear(&ribtn, 0.25, 0)?;
    view_map.insert((ribtn.act().aid()?, ribtn.id()?), "ribtn");

    //    // Right Button
    //    let r_btn_view = act.new_button(layout_linear.id()?, false, "r_btn".to_string())?;
    //    let r_abv = act.gen_view(&r_btn_view).unwrap();
    ////dbg!(&r_abv);
    //act.vi_clickable(r_abv.clone(), false)?;
    //act.vi_click_event(r_abv.clone(), true)?;
    ////act.vi_touch_event(r_abv.clone(), true)?;
    //act.vi_bg(r_abv.clone(), 0x00000000)?;
    //act.vi_fg(r_abv.clone(), 0x00000000)?;
    //act.set_layout_linear(r_abv.clone(), 0.25, 0)?;
    //view_map.insert((r_abv.act().aid()?, r_abv.id()?), "r_btn");

    // LinearLayout - ProgressBar Wrapper
    let layout_linear_pg = act.new_layout_linear(&layout_frame, false)?;

    // Space
    let space_view = act.new_space(&layout_linear_pg)?;
    act.set_layout_linear(&space_view, 1.0, 0)?;

    // Left TextView
    let l_text = Text::new(&act)
        .set_data(act.gen_create().unwrap().set_parent(layout_linear_pg.id()?))
        .set_selectable_text(false)
        .set_clickable_links(false)
        .set_text("hi".to_string())
        .conn()?;
    act.set_layout_linear(&l_text, 0.0, 0)?;

    // ProgressBar
    let pg = act.new_progress_bar(&layout_linear_pg)?;
    pg.vi_click_event(true)?;
    act.set_layout_linear(&pg, 0.2, 0)?;
    view_map.insert((pg.act().aid()?, pg.id()?), "pg");

    layout_linear_pg.vi_visible(Visibility::Hidden)?;

    {
        play::DATA.get_or_init(|| Data::new(act.clone(), layout_linear_pg.clone()));
    }

    drop(view_map);
    drop(tmp);

    unsafe {
        buffer_res.mmap()?;
    }

    // ============== UI ==============
    //let phy_rect: PhyRect = rect(0, 0, w as i32, h as i32);
    //let start_time = SystemTime::now();
    //let (x, y) = &mut (0, 0);

    // ============== Main ==============
    while !play::FLAG_EXIT.load(std::sync::atomic::Ordering::Relaxed) {
        //for f in (0..300).step_by(70) {
        //    let r: PhyRect = rect(x, y, 100, 100);
        //    r.fill(&phy_rect, &mut vec_rgba);
        //
        //    let r: PhyRect = rect(x, y + f, 100, 100);
        //    r.fill(&phy_rect, &mut vec_rgba);
        //
        //    let r: PhyRect = rect(x, y + f + f, 100, 100);
        //    r.fill(&phy_rect, &mut vec_rgba);
        //}

        {
            let mut frame = arc_buffer.write().unwrap();
            if !frame.is_synced {
                continue;
            }

            let data = frame.inner.mut_data();
            //buffer_res.mmap_flush_with_swap(data)?;

            //let offset_x = 100;
            //let offset_y = 100;
            //let dst_w = width / 2;
            //let dst_h = height / 2;
            //for y in (0..dst_h) {
            //    for x in (0..dst_w * 4) {
            //        let y = y + offset_y;
            //        let x = x + offset_x;
            //        let idx = (y * width * 4) + x;
            //        let i = idx as usize;
            //        data[i] = 255;
            //    }
            //}

            buffer_res.mmap_flush_with_swap(data)?;
            frame.is_synced = false;

            l_text.update(format!(
                "{l} - {r}",
                l = frame.fmt_cur_time(),
                r = frame.fmt_total_time(),
            ))?;

            // ProgressBar
            let n = (frame.cur_time as f64 / frame.total_time as f64) * 100.0;
            pg.set(n as u32)?;
        }
        buffer_res.blit(&act)?;
        img.refresh(&img)?;
        act.config_set_no_bar()?;

        // TODO: do it better
        sleep(Duration::from_nanos(100));
    }

    // exit window
    act.close();

    Ok(())
}
