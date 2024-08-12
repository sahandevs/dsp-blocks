#![allow(non_upper_case_globals)]

use std::time::Duration;

use raylib::prelude::*;
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
    let mut audio = RaylibAudio::init_audio_device()?;

    let note_a = signal::create_sinusoid(440.0, 0f32, Duration::from_millis(100));

    let view_offset = (Duration::from_millis(0).as_secs_f32() * signal::SR as f32) as usize;
    let view_size = (Duration::from_millis(10).as_secs_f32() * signal::SR as f32) as usize;

    let view = &note_a[view_offset..view_offset + view_size];

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        draw_block_audio_out(&mut audio, &mut d);

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
    let step = (wave_in.len() as f64 / 50 as f64).ceil() as usize;
    let wave: Vec<f32> = wave_in.iter().step_by(step).take(50).cloned().collect();
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

fn draw_block_audio_out(_audio: &mut RaylibAudio, d: &mut RaylibDrawHandle) {
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
