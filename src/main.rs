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
    let (input, mut system) = setups::playground::create_playground_blocks()?;

    let (_, texture) = {
        let mut draw_context = DrawContext {
            thread: &thread,
            rl: &mut rl,
        };

        system.process_and_visualize(input.clone(), &mut draw_context)
    };

    while !rl.window_should_close() {
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
    }
    Ok(())
}
