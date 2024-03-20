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
    thread::{self, sleep_ms},
};
use tgui::{items::Visibility, View, ViewSet, *};
use tgui_player::{
    play::{self, Args, Data, WrapBuffer},
    rect, PhyRect, Rect,
};

fn main() -> Res<()> {
    // ============== Gstreamer ==============
    let args = Args::new();
    let arc_buffer = Arc::new(RwLock::new(WrapBuffer::new(Buffer::zero(0, 0)?, (0, 0))));
    dbg!(&args);

    let uri = args.uri().to_string();
    let buffer = arc_buffer.clone();
    thread::spawn(|| {
        let pipe = play::create_pipeline(buffer, uri);
        play::main(pipe.pipeline, args).unwrap();
    });

    // ============== Window ==============
    while arc_buffer.read().unwrap().inner.width == 0 {
        sleep_ms(10);
    }

    let mut view_map = HashMap::<(i32, i32), &'static str>::new();

    // Layout:
    //
    // <FrameLayout>
    //   <LinearLayout>
    //     <Button on-left />
    //     <Button on-right />
    //   </LinearLayout>
    //
    //   <LinearLayout>
    //     <Image on-center />
    //   </LinearLayout>
    // </FrameLayout>

    let tgui = Tgui::new()?.conn()?;
    let act = Activity::new().conn(&tgui)?;
    tgui.config_keep_screen_on(&act)?;

    // FrameLayout
    let data = act.gen_create().unwrap().set_parent(-1);
    let layout_frame = tgui.new_layout_frame(data)?;

    // LinearLayout - Image
    let data = act.gen_create().unwrap().set_parent(layout_frame.get_id()?);
    let layout_linear_img = tgui.new_layout_linear(data, true)?;
    let alliv = act.gen_view(layout_linear_img.res()).unwrap();

    // ImageView
    let tmp = arc_buffer.read().unwrap().clone();
    let (width, height) = tmp.size;
    let mut buffer_res = tgui.new_buffer(&tmp.inner)?;
    let data = act
        .gen_create()
        .unwrap()
        .set_parent(layout_linear_img.get_id()?);
    let img = Img::new().set_data(data).conn(&tgui)?;
    let aiv = act.gen_view(img.res()).unwrap();

    tgui.buffer_set(act.get_id()?, &img, &buffer_res)?;
    tgui.view_set_clickable(aiv.clone(), false)?;
    tgui.view_set_click_event(aiv.clone(), true)?;
    //tgui.view_set_touch_event(aiv.clone(), true)?;

    // LinearLayout - Button
    let data = act.gen_create().unwrap().set_parent(layout_frame.get_id()?);
    let layout_linear = tgui.new_layout_linear(data, true)?;

    // Left Button
    let data = act
        .gen_create()
        .unwrap()
        .set_parent(layout_linear.get_id()?);
    let l_btn_view = tgui.new_button(data, false, "l_btn".to_string())?;
    let l_abv = act.gen_view(&l_btn_view).unwrap();
    tgui.view_set_clickable(l_abv.clone(), false)?;
    tgui.view_set_touch_event(l_abv.clone(), true)?;
    tgui.view_set_bg(l_abv.clone(), 0x00000000)?;
    tgui.view_set_fg(l_abv.clone(), 0x00000000)?;
    tgui.set_layout_linear(l_abv.clone(), 0.25, 0)?;
    view_map.insert((l_abv.aid, l_abv.id), "l_btn");

    // Space
    let data = act
        .gen_create()
        .unwrap()
        .set_parent(layout_linear.get_id()?)
        .set_v(Visibility::Hidden);
    let space_view = tgui.new_space(data)?;
    let asv = act.gen_view(&space_view).unwrap();
    tgui.set_layout_linear(asv.clone(), 1.0 - 0.25 * 2.0, 0)?;

    // Right Button
    let data = act
        .gen_create()
        .unwrap()
        .set_parent(layout_linear.get_id()?);
    let r_btn_view = tgui.new_button(data, false, "r_btn".to_string())?;
    let r_abv = act.gen_view(&r_btn_view).unwrap();
    dbg!(&r_abv);
    tgui.view_set_clickable(r_abv.clone(), false)?;
    tgui.view_set_touch_event(r_abv.clone(), true)?;
    tgui.view_set_bg(r_abv.clone(), 0x00000000)?;
    tgui.view_set_fg(r_abv.clone(), 0x00000000)?;
    tgui.set_layout_linear(r_abv.clone(), 0.25, 0)?;
    view_map.insert((r_abv.aid, r_abv.id), "r_btn");

    {
        play::DATA.get_or_init(|| Data::new(tgui.clone(), act.clone()));
    }

    unsafe {
        buffer_res.mmap()?;
    }

    // ============== UI ==============
    //let phy_rect: PhyRect = rect(0, 0, w as i32, h as i32);
    //let start_time = SystemTime::now();
    //let (x, y) = &mut (0, 0);

    // ============== Main ==============
    loop {
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

            // TODO: progress bar
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
        }
        tgui.buffer_blit(buffer_res.bid)?;
        img.refresh(&tgui, aiv.clone())?;
        tgui.config_set_no_bar(&act)?;

        //sleep_ms(1);
    }
}
