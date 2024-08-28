#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use dsp::Block;
use raylib::prelude::*;
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

    rl.set_target_fps(60);

    let mut cam = Camera2D {
        offset: Vector2::new(W as f32 / 2.0, H as f32 / 2.0),
        target: Vector2::new(W as f32 / 2.0, H as f32 / 2.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    while !rl.window_should_close() {
        // zoom
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            let zoom_increment = 0.05;
            cam.zoom += wheel as f32 * zoom_increment;
            cam.zoom = cam.zoom.max(0.1).min(3.0);

            let mouse_pos = rl.get_mouse_position();
            let mouse_world_pos = rl.get_screen_to_world2D(mouse_pos, cam);
            let delta = mouse_world_pos - cam.target;
            cam.target +=
                delta * (1.0 - (1.0 / (cam.zoom / (cam.zoom + wheel as f32 * zoom_increment))));
        }

        // panning
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            let delta = rl.get_mouse_delta().scale_by(-1.0 / cam.zoom);
            cam.target += delta;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(vis::BG_COLOR);
        {
            let mut d = d.begin_mode2D(cam);
            if let Some(x) = texture.as_simple_texture() {
                let mut d = d.begin_blend_mode(BlendMode::BLEND_ALPHA);
                d.draw_texture_rec(
                    &x,
                    Rectangle {
                        width: x.width() as _,
                        height: -x.height() as _,
                        ..Default::default()
                    },
                    Vector2::new(50.0, 50.0),
                    Color::WHITE,
                );
            }
        }

        // Draw UI elements
        d.draw_text(&format!("Zoom: {:.2}", cam.zoom), 10, 10, 20, Color::WHITE);
    }
    Ok(())
}
