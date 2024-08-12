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
    }
    Ok(())
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
