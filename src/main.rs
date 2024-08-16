#![allow(non_upper_case_globals)]

use std::time::Duration;

use raylib::prelude::*;
use rodio::{OutputStream, Sink, Source};
pub mod signal;

const W: i32 = 1080;
const H: i32 = 720;
const T: f32 = 3f32;

fn main() -> anyhow::Result<()> {
    let (mut rl, thread) = raylib::init()
        .msaa_4x()
        .size(W, H)
        .title("DSP Blocks")
        .build();

    let total_dur = Duration::from_millis(1000);
    let note_a = signal::create_sinusoid(440.0, 0f32, total_dur);
    let note_b = signal::create_sinusoid(493.0, 0f32, total_dur);
    let x_t = signal::mix_waves(&note_a, &note_b);

    let mut view_dur = Duration::from_millis(10).as_secs_f32();
    let mut view_offset_dur = Duration::from_millis(25).as_secs_f32();

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let source =
        rodio::buffer::SamplesBuffer::new(1, signal::SR as u32, x_t.clone()).repeat_infinite();
    let total_duration = x_t.len() as f32 / signal::SR as f32;
    sink.append(source);
    sink.set_volume(0.2);
    sink.play();

    while !rl.window_should_close() {
        let view_offset = (view_offset_dur * signal::SR as f32) as usize;
        let view_size = (view_dur * signal::SR as f32) as usize;
        let full_view = &x_t;
        let view = &full_view[view_offset..view_offset + view_size];
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        draw_block_audio_out(&mut d);

        let current_sound_pos = (sink.get_pos().as_secs_f32() / total_duration) % total_duration;

        // overview
        let overview_w = (W as f32) - 20f32;
        let overview_rec = Rectangle {
            x: 10f32,
            y: 10f32,
            width: overview_w,
            height: 100f32,
        };
        draw_wave_box(&mut d, overview_rec, &full_view);

        d.draw_line_ex(
            Vector2::new(
                overview_rec.x + current_sound_pos * overview_w,
                overview_rec.y,
            ),
            Vector2::new(
                overview_rec.x + current_sound_pos * overview_w,
                overview_rec.y + overview_rec.height,
            ),
            1f32,
            Color::RED,
        );

        let view_select_rec = Rectangle {
            height: 100f32,
            x: 10f32 + overview_w * (view_offset as f32 / full_view.len() as f32),
            y: 10f32,
            width: overview_w * (view.len() as f32 / full_view.len() as f32),
        };

        let mut view_select_color = Color::WHITE.alpha(0.2);
        if view_select_rec.check_collision_point_rec(d.get_mouse_position()) {
            d.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_POINTING_HAND);

            view_select_color = Color::WHITE.alpha(0.3);

            let wheel = d.get_mouse_wheel_move();
            if wheel != 0f32 {
                view_dur += wheel * 0.001f32;
            }
        }

        if overview_rec.check_collision_point_rec(d.get_mouse_position()) {
            d.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_POINTING_HAND);
            if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                let x_rel = (d.get_mouse_position().x - overview_rec.x) / overview_rec.width;
                view_offset_dur = total_dur.as_secs_f32() * x_rel;
            }
        } else {
            d.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_DEFAULT);
        }

        view_dur = view_dur
            .min(total_dur.as_secs_f32() - view_offset_dur)
            .max(Duration::from_millis(1).as_secs_f32());

        d.draw_rectangle_rec(view_select_rec, view_select_color);

        draw_wave_box(
            &mut d,
            Rectangle {
                x: 50f32,
                y: 300f32,
                height: 100f32,
                width: 200f32,
            },
            &view,
        );
    }
    Ok(())
}

fn draw_wave_box(d: &mut RaylibDrawHandle, rec: Rectangle, wave_in: &[f32]) {
    let n = (rec.width as usize * rec.width as usize / 2) / 200;
    let step = (wave_in.len() as f64 / n as f64).ceil() as usize;
    let wave: Vec<f32> = wave_in.iter().step_by(step).take(n).cloned().collect();
    let n_samples = wave.len();
    let max = wave
        .iter()
        .map(|x| x.abs())
        .reduce(f32::max)
        .unwrap_or_default();

    let spacing = rec.width / (n_samples + 1) as f32;

    d.draw_rectangle_rec(rec, Color::BLACK);
    let center_y = rec.y + rec.height / 2f32;
    // center line

    d.draw_line(
        rec.x as _,
        center_y as _,
        (rec.x + rec.width) as _,
        center_y as _,
        Color::GRAY,
    );
    let mut offset = spacing;
    let l_w = (5f32 * (T * 1.5 / n_samples as f32)).max(1f32);
    for sample in wave {
        let amp = sample / max;
        let y = center_y - amp * (rec.height / 2.5f32);
        const COLOR: Color = Color::BLUEVIOLET;

        d.draw_line_ex(
            Vector2::new(rec.x + offset, center_y),
            Vector2::new(rec.x + offset, y),
            l_w,
            COLOR,
        );

        d.draw_rectangle_rec(
            Rectangle {
                x: rec.x + offset - l_w,
                y,
                width: l_w * 2f32,
                height: l_w * 2f32,
            },
            COLOR,
        );

        offset += spacing;
    }

    // borders
    d.draw_rectangle_lines_ex(rec, T / 1.5f32, Color::WHITE);
}

fn draw_block_audio_out(d: &mut RaylibDrawHandle) {
    const h: i32 = 40;
    const p: i32 = 10;
    const w: i32 = 20 + p * 4;

    // TODO: https://www.raylib.com/examples/audio/loader.html?name=audio_raw_stream

    let x = (W - w - p - 10) as f32;
    let y = (H - h - p - 10) as f32;

    let rec = Rectangle {
        x: x,
        y: y,
        width: w as _,
        height: h as _,
    };
    d.draw_rectangle_lines_ex(rec, T, Color::WHITE);
    d.draw_text("Out", x as i32 + p, y as i32 + p, 20, Color::WHITE);
}
