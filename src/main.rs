#![allow(non_upper_case_globals)]

use dsp::Block;
use raylib::prelude::*;
use rodio::{OutputStream, Sink, Source};
use vis::DrawContext;
pub mod dsp;
pub mod setups;
pub mod vis;

const W: i32 = 1080;
const H: i32 = 720;

fn main() -> anyhow::Result<()> {
    let (mut rl, thread) = raylib::init()
        .msaa_4x()
        .size(W, H)
        .title("DSP Blocks")
        .build();

    // select a system
    let (input, mut system) = setups::playground::create_playground_blocks();

    let (x_t, texture) = {
        let mut draw_context = DrawContext {
            thread: &thread,
            rl: &mut rl,
        };

        system.process_and_visualize(input.clone(), &mut draw_context)
    };

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let source =
        rodio::buffer::SamplesBuffer::new(1, dsp::SR as u32, x_t.clone()).repeat_infinite();
    let total_duration = x_t.len() as f32 / dsp::SR as f32;
    sink.append(source);
    sink.set_volume(0.2);
    sink.play();

    while !rl.window_should_close() {
        let full_view = &x_t;

        let mut d = rl.begin_drawing(&thread);

        if let Some(x) = texture.as_simple_texture() {
            let mut d = d.begin_blend_mode(BlendMode::BLEND_ALPHA);
            d.draw_texture_rec(
                &x,
                Rectangle {
                    width: x.width() as _,
                    height: -x.height() as _,
                    ..Default::default()
                },
                Vector2::new(150f32, 150f32),
                Color::WHITE,
            );
        }

        d.clear_background(vis::BG_COLOR);
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
    }
    Ok(())
}

fn draw_wave_box(d: &mut RaylibDrawHandle, rec: Rectangle, wave_in: &[f32]) {
    let n = (rec.width as usize * rec.width as usize) / 200;
    let step = (wave_in.len() as f64 / n as f64).ceil() as usize;
    let wave: Vec<f32> = wave_in.iter().step_by(step).take(n).cloned().collect();
    let n_samples = wave.len();
    let max = wave
        .iter()
        .map(|x| x.abs())
        .reduce(f32::max)
        .unwrap_or_default();

    let spacing = rec.width / (n_samples + 1) as f32;

    d.draw_rectangle_rec(rec, vis::BG_COLOR);
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
    let l_w = (5f32 * (vis::T * 1.5 / n_samples as f32)).max(1f32);

    let get_y = |sample: f32| (center_y - (sample / max) * (rec.height / 2.5f32));
    let mut last_point = get_y(wave[0]);
    for sample in wave {
        let y = get_y(sample);
        const COLOR: Color = Color::BLUEVIOLET;

        d.draw_line_ex(
            Vector2::new(rec.x + offset - spacing, last_point),
            Vector2::new(rec.x + offset, y),
            l_w,
            COLOR,
        );

        last_point = y;

        // d.draw_circle_lines(
        //     (rec.x + offset - l_w + 2f32) as _,
        //     y as _,
        //     2f32,
        //     COLOR,
        // );

        offset += spacing;
    }

    // borders
    d.draw_rectangle_lines_ex(rec, vis::T / 1.5f32, Color::WHITE);
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
    d.draw_rectangle_lines_ex(rec, vis::T, Color::WHITE);
    d.draw_text("Out", x as i32 + p, y as i32 + p, 20, Color::WHITE);
}
