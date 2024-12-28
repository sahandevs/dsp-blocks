use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::dsp;
use crate::dsp::Wave;
use crate::graph::{Block, DInto};
use raylib::prelude::*;

pub const BOX_SIZE: f32 = 55f32;
pub const BORDER_COLOR: Color = Color::WHITE;
pub const BG_COLOR: Color = Color::BLACK;
pub const TEXT_COLOR: Color = Color::WHITE;
pub const T: f32 = 3f32;

pub const LINE_COLORS: [Color; 4] = [
    Color::BLUEVIOLET,
    Color::ROYALBLUE,
    Color::ORANGE,
    Color::BROWN,
];

pub struct DrawContext<'a, 'b> {
    pub thread: &'a RaylibThread,
    pub rl: &'b mut RaylibHandle,
}

impl<'a, 'b> DrawContext<'a, 'b> {
    pub fn get_texture(&mut self, width: u32, height: u32) -> RenderTexture2D {
        self.rl
            .load_render_texture(self.thread, width, height)
            .unwrap()
    }
}

pub enum VisualizeResult {
    None,
    SimpleTexture(RenderTexture2D),
    Block {
        texture: RenderTexture2D,
        input_connections: Vec<Vector2>,
        output_connections: Vec<Vector2>,
    },
}

#[derive(Debug)]
pub struct Identity;
impl<I> Block<I> for Identity {
    type Output = I;

    fn process(&mut self, input: I) -> Self::Output {
        input
    }

    fn process_and_visualize(
        &mut self,
        input: I,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let out = self.process(input);

        let tx = context.get_texture(BOX_SIZE as _, BOX_SIZE as _);

        let center = Vector2::new(BOX_SIZE, BOX_SIZE) / 2f32;
        (
            out,
            VisualizeResult::Block {
                texture: tx,
                input_connections: vec![Vector2::new(center.x, center.y)],
                output_connections: vec![Vector2::new(center.x, center.y)],
            },
        )
    }
}

impl VisualizeResult {
    pub fn as_simple_texture(&self) -> Option<&RenderTexture2D> {
        match self {
            VisualizeResult::None => None,
            VisualizeResult::SimpleTexture(x) => Some(x),
            VisualizeResult::Block {
                texture,
                input_connections: _,
                output_connections: _,
            } => Some(texture),
        }
    }
}

pub fn draw_border(d: &mut impl RaylibDraw, rec: Rectangle) {
    d.draw_rectangle_lines_ex(rec, T / 1.5f32, Color::WHITE);
}
pub fn draw_wave(
    d: &mut impl RaylibDraw,
    rec: Rectangle,
    wave_in: &[f32],
    color: Color,
    spacing: f32,
) {
    let n = rec.width.trunc() as usize;
    let step = (wave_in.len() as f64 / n as f64).ceil() as usize;

    // Max-pooling downsampling
    // TODO: maybe most-diff pooling results are better?
    let wave: Vec<f32> = (0..n)
        .map(|i| {
            let start = i * step;
            let end = (start + step).min(wave_in.len());
            if start < end {
                wave_in[start..end].iter().fold(
                    0f32,
                    |max, &x| {
                        if max.abs() > x.abs() {
                            max
                        } else {
                            x
                        }
                    },
                )
            } else {
                0f32 // slice is empty
            }
        })
        .collect();

    let n_samples = wave.len();

    let max = wave
        .iter()
        .map(|x| x.abs())
        .reduce(f32::max)
        .unwrap_or_default();

    let spacing = if spacing == 0f32 {
        rec.width / (n_samples + 1) as f32
    } else {
        spacing
    };

    let center_y = (rec.y + rec.height / 2f32).trunc();

    let mut offset = spacing.trunc();

    // draw y guide lines.
    if max.trunc() > 1f32 {
        let p_guideline = rec.height / max;

        for i in 0..max.abs() as usize {
            d.draw_line_ex(
                Vector2::new(0f32, center_y + p_guideline * i as f32),
                Vector2::new(rec.width, center_y + p_guideline * i as f32),
                1f32,
                Color::GRAY,
            );
            d.draw_line_ex(
                Vector2::new(0f32, center_y - p_guideline * i as f32),
                Vector2::new(rec.width, center_y - p_guideline * i as f32),
                1f32,
                Color::GRAY,
            );
        }
    }

    let get_y = |sample: f32| (center_y - (sample / max) * (rec.height / 2.5f32)).trunc();
    let mut last_point = get_y(wave[0]);
    for sample in wave {
        let y = get_y(sample);

        // vertical lines
        if false {
            d.draw_line_ex(
                Vector2::new(rec.x + offset, center_y),
                Vector2::new(rec.x + offset, y),
                1f32,
                color,
            );
        }

        // Draw connecting line
        d.draw_line_ex(
            Vector2::new(rec.x + offset - spacing, last_point),
            Vector2::new(rec.x + offset, y),
            1f32,
            color,
        );

        last_point = y;
        offset += spacing;
    }
}

pub fn draw_wave_box(
    d: &mut impl RaylibDraw,
    rec: Rectangle,
    wave_in: &[f32],
    color: Color,
    spacing: f32,
) {
    // center line
    d.draw_line_ex(
        Vector2::new(0f32, (rec.height / 2f32).trunc()),
        Vector2::new(rec.width, (rec.height / 2f32).trunc()),
        1f32,
        Color::SILVER,
    );
    draw_wave(d, rec, wave_in, color, spacing);
    draw_border(d, rec);
}

pub fn draw_simple_bock(d: &mut impl RaylibDraw, text: &str) -> Vector2 {
    let center = (BOX_SIZE / 2f32).trunc();
    let h = (BOX_SIZE / 2f32) + T;
    let num_lines = text.split("\n").count();
    d.draw_text(
        text,
        (T + 2f32) as _,
        ((h + 2f32) - ((num_lines - 1) as f32 * 10f32)).trunc() as _,
        3,
        TEXT_COLOR,
    );
    draw_border(
        d,
        Rectangle {
            height: BOX_SIZE,
            width: BOX_SIZE,
            ..Default::default()
        },
    );

    Vector2::new(center, center)
}

// visual/analyze blocks

pub fn visualize_simple_box<O>(
    context: &mut DrawContext,
    text: &str,
    out: O,
) -> (O, VisualizeResult) {
    let mut tx = context.get_texture(BOX_SIZE as _, BOX_SIZE as _);
    tx.set_texture_filter(
        context.thread,
        raylib::ffi::TextureFilter::TEXTURE_FILTER_ANISOTROPIC_16X,
    );
    let mut d = context.rl.begin_drawing(context.thread);
    let mut d = d.begin_texture_mode(context.thread, &mut tx);

    let center = draw_simple_bock(&mut d, text);
    drop(d);
    (
        out,
        VisualizeResult::Block {
            texture: tx,
            input_connections: vec![Vector2::new(0f32, center.y)],
            output_connections: vec![Vector2::new(BOX_SIZE, center.y)],
        },
    )
}

pub struct AudioSink {
    buffer: Arc<Mutex<Wave>>,
    // client: jack::AsyncClient<(), jack::contrib::ClosureProcessHandler<(), F>>,
}

impl Debug for AudioSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioSink").finish()
    }
}

impl AudioSink {
    pub fn try_default() -> anyhow::Result<Self> {
        let (client, _status) =
            jack::Client::new("dsp-block_audio-sink", jack::ClientOptions::default())?;

        assert_eq!(
            client.sample_rate(),
            dsp::SR,
            "dsp-blocks only work on sample rate {} but got {} from jack",
            dsp::SR,
            client.sample_rate()
        );

        struct State {
            buffer: Arc<Mutex<Wave>>,
            cursor: usize,
            port: jack::Port<jack::AudioOut>,
        }
        let out = client.register_port("output", jack::AudioOut::default())?;

        let mut state = State {
            buffer: Arc::new(Default::default()),
            cursor: 0,
            port: out,
        };
        let buffer = state.buffer.clone();

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let g = {
                let this = state.buffer.lock();
                match this {
                    Ok(t) => t,
                    Err(e) => std::process::exit(1),
                }
            };
            let client_buffer = g.as_slice();

            let port_buffer = state.port.as_mut_slice(ps);

            let select = &client_buffer
                [state.cursor..(state.cursor + port_buffer.len()).min(client_buffer.len())];
            
            if select.len() == 0 {
                return jack::Control::Continue;
            }
            println!("xxx");
            if select.len() == port_buffer.len() {
                println!("{}", select[50]);
                port_buffer.copy_from_slice(select);

            }
            
            let remaining =
            port_buffer.len() as isize - (client_buffer.len() - state.cursor) as isize;
            println!("xxx {}", remaining);

            if remaining <= 0 {
                state.cursor += port_buffer.len();
            } else {
                state.cursor = 0;
                let remaining_out_buffer = &mut port_buffer[remaining as usize..];
                remaining_out_buffer.copy_from_slice(
                    &client_buffer[state.cursor
                        ..(state.cursor + remaining_out_buffer.len()).min(client_buffer.len())],
                );
                state.cursor += remaining as usize;
            }

            jack::Control::Continue
        };
        let process = jack::contrib::ClosureProcessHandler::new(Box::new(process_callback));

        let x = client.activate_async((), process)?;
        // FIXME: handle cleanup
        std::mem::forget(x);
        Ok(AudioSink { buffer })
    }
}

impl Block<Wave> for AudioSink {
    type Output = Wave;

    fn process(&mut self, input: Wave) -> Self::Output {
        self.buffer
            .lock()
            .unwrap()
            .extend_from_slice(input.as_slice());
        // let total_duration = input.len() as f32 / dsp::SR as f32;
        // let current_sound_pos = (sink.get_pos().as_secs_f32() / total_duration) % total_duration;

        input
    }

    fn process_and_visualize(
        &mut self,
        input: Wave,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let out = self.process(input);
        visualize_simple_box(context, "Sink", out)
    }
}

#[derive(Debug, tidy_builder::Builder)]
pub struct WaveView<const N: usize> {
    #[builder(value = default)]
    pub t: WaveViewType,

    #[builder(value = false)]
    pub is_hovering: bool,
}

impl<const N: usize> WaveView<N> {
    pub fn grow() -> Self {
        WaveView::builder().t(WaveViewType::Grow).build()
    }

    pub fn small() -> Self {
        WaveView::builder().t(WaveViewType::Small).build()
    }
}

#[derive(Debug, Default)]
pub enum WaveViewType {
    Grow,
    #[default]
    Small,
}

impl<I: DInto<[Wave; N]>, const N: usize> Block<I> for WaveView<N> {
    type Output = I;

    fn process(&mut self, input: I) -> Self::Output {
        input
    }

    fn process_and_visualize(
        &mut self,
        input: I,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let out: [Wave; N] = input.into();

        let unit = (if self.is_hovering { 10f32 } else { 2f32 }) * BOX_SIZE;

        let stat_font_size: f32 = 13f32;
        let stats_height: f32 = N as f32 * stat_font_size;

        let rec = match self.t {
            WaveViewType::Grow => {
                // unit per SR
                Rectangle {
                    width: ((out.iter().map(|x| x.len()).max().unwrap_or_default() / dsp::SR) + 1)
                        as f32
                        * (unit * 2f32),
                    height: 50f32,
                    x: 0f32,
                    y: 0f32,
                }
            }
            WaveViewType::Small => {
                // fit everything in unit
                Rectangle {
                    width: unit,
                    height: 50f32,
                    x: 0f32,
                    y: 0f32,
                }
            }
        };

        let mut tx = context.get_texture(rec.width as _, rec.height as u32 + stats_height as u32);
        tx.set_texture_filter(
            context.thread,
            raylib::ffi::TextureFilter::TEXTURE_FILTER_ANISOTROPIC_16X,
        );
        let mut d = context.rl.begin_drawing(context.thread);
        let mut d = d.begin_texture_mode(context.thread, &mut tx);

        for (i, wave) in out.iter().enumerate() {
            d.draw_line_ex(
                Vector2::new(0f32, (rec.height / 2f32).trunc()),
                Vector2::new(rec.width, (rec.height / 2f32).trunc()),
                1f32,
                Color::GRAY,
            );

            let inner_box = Rectangle {
                width: rec.width - 10f32,
                height: rec.height,
                x: 5f32,
                y: 0f32,
            };
            // FIXME: draw_wave scales the y values based on y resulting all waveforms being normalized.
            draw_wave(
                &mut d,
                inner_box,
                &wave,
                LINE_COLORS[i],
                if matches!(self.t, WaveViewType::Grow) {
                    0f32
                } else {
                    1f32
                },
            );
        }
        draw_border(&mut d, rec);

        for i in 0..N {
            let mean = out[i].iter().fold(0f32, |acc, x| acc + x) / out[i].len() as f32;
            let std = (out[i]
                .iter()
                .map(|x| (*x - mean).powi(2))
                .fold(0f32, |acc, x| acc + x)
                / out[i].len() as f32)
                .sqrt();
            d.draw_text(
                format!("u={mean:.3}, s={std:.3}").as_str(),
                rec.x as _,
                (rec.height + (stat_font_size * i as f32)) as _,
                stat_font_size as _,
                LINE_COLORS[i],
            );
        }

        drop(d);
        (
            DInto::from(out),
            VisualizeResult::Block {
                texture: tx,
                input_connections: vec![Vector2::new(0f32, rec.height / 2f32)],
                output_connections: vec![Vector2::new(rec.width, rec.height / 2f32)],
            },
        )
    }

    // fn on_hover(
    //     &mut self,
    //     _pos: Vector2,
    //     context: &mut crate::control::ControlContext,
    // ) -> crate::control::ControlResult {
    //     if !self.is_hovering {
    //         self.is_hovering = true;
    //         context.is_dirty = true;
    //     }
    //     crate::control::ControlResult::Passthrough
    // }

    // fn on_unhover(
    //     &mut self,
    //     context: &mut crate::control::ControlContext,
    // ) -> crate::control::ControlResult {
    //     if self.is_hovering {
    //         self.is_hovering = false;
    //         context.is_dirty = true;
    //     }
    //     crate::control::ControlResult::Passthrough
    // }
}
