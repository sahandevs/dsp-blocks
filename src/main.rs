#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use std::env;

use control::ControlContext;
use graph::Block;
use raylib::prelude::*;
use vis::DrawContext;
pub mod control;
pub mod dsp;
pub mod graph;
pub mod setups;
pub mod vis;
pub mod wav;

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
    let mut texture;

    macro_rules! redraw {
        () => {{
            let mut draw_context = DrawContext {
                thread: &thread,
                rl: &mut rl,
            };

            let (_, t) = system.process_and_visualize(input.clone(), &mut draw_context);
            texture = t;
        }};
    }

    redraw!();

    rl.set_target_fps(60);

    let mut cam = Camera2D {
        offset: Vector2::new(W as f32 / 2.0, H as f32 / 2.0),
        target: Vector2::new(W as f32 / 2.0, H as f32 / 2.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    #[allow(unused_mut)]
    let mut debug = format!("");

    let sys_pos = Vector2::new(50.0, 100.0);

    let mut last_time_hover = false;
    while !rl.window_should_close() {
        let mouse_pos = rl.get_mouse_position();
        let mouse_world_pos = rl.get_screen_to_world2D(mouse_pos, cam);
        // zoom
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            let zoom_increment = 0.05;
            cam.zoom += wheel as f32 * zoom_increment;
            cam.zoom = cam.zoom.max(0.1).min(3.0);

            let delta = mouse_world_pos - cam.target;
            cam.target +=
                delta * (1.0 - (1.0 / (cam.zoom / (cam.zoom + wheel as f32 * zoom_increment))));
        }

        // panning
        let mouse_delta = rl.get_mouse_delta();
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            let delta = mouse_delta.scale_by(-1.0 / cam.zoom);
            cam.target += delta;
        }

        cam.target.x = cam.target.x.trunc();
        cam.target.y = cam.target.y.trunc();

        // interactive controls
        let mut needs_total_redraw = false;
        if let Some(x) = texture.as_simple_texture() {
            let tx_rec = Rectangle {
                width: x.width() as _,
                height: x.height() as _,
                x: sys_pos.x,
                y: sys_pos.y,
            };
            let mut control_ctx = ControlContext {
                thread: &thread,
                rl: &mut rl,
                is_dirty: false,
            };
            if tx_rec.check_collision_point_rec(mouse_world_pos) {
                if mouse_delta.length() > 0f32 {
                    system.on_hover(mouse_world_pos - sys_pos, &mut control_ctx);
                    last_time_hover = true;
                }
            } else if last_time_hover {
                system.on_unhover(&mut control_ctx);
                last_time_hover = false;
            }
            needs_total_redraw = control_ctx.is_dirty;
        }

        if needs_total_redraw {
            println!("did redraw");
            redraw!();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(vis::BG_COLOR);
        {
            let mut d = d.begin_mode2D(cam);
            if let Some(x) = texture.as_simple_texture() {
                x.set_texture_filter(
                    &thread,
                    raylib::ffi::TextureFilter::TEXTURE_FILTER_ANISOTROPIC_16X,
                );
                let mut d = d.begin_blend_mode(BlendMode::BLEND_ALPHA);
                d.draw_texture_rec(
                    &x,
                    Rectangle {
                        width: x.width() as _,
                        height: -x.height() as _,
                        ..Default::default()
                    },
                    sys_pos,
                    Color::WHITE,
                );
            }
        }

        // Draw UI elements
        if env::var("NO_TEXT").unwrap_or_default().is_empty() {
            d.draw_text(&format!("Zoom: {:.2}", cam.zoom), 10, 10, 20, Color::WHITE);
            d.draw_text(&format!("Debug: {}", &debug), 10, 30, 20, Color::WHITE);
        }
    }
    Ok(())
}
