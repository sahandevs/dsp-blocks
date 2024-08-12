#![allow(non_upper_case_globals)]

use raylib::prelude::*;

const W: i32 = 1080;
const H: i32 = 720;
const T: f32 = 3f32;

fn main() -> anyhow::Result<()> {
    let (mut rl, thread) = raylib::init().size(W, H).title("DSP Blocks").build();
    let mut audio = RaylibAudio::init_audio_device()?;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        draw_block_audio_out(&mut audio, &mut d);

        draw_wave_box(
            &mut d,
            Rectangle {
                x: 50f32,
                y: 100f32,
                height: 100f32,
                width: 200f32,
            },
            &[],
        );
    }
    Ok(())
}

fn draw_wave_box(d: &mut RaylibDrawHandle, rec: Rectangle, wave: &[f32]) {
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
        let y = amp * (rec.height / 2.5f32) + center_y;
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
