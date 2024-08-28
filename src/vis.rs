use crate::dsp;
use crate::dsp::Block;
use crate::dsp::Wave;
use raylib::prelude::*;
use rodio::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};

pub const BOX_SIZE: f32 = 50f32;
pub const BORDER_COLOR: Color = Color::WHITE;
pub const BG_COLOR: Color = Color::BLACK;
pub const TEXT_COLOR: Color = Color::WHITE;
pub const T: f32 = 3f32;

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

pub fn draw_wave(d: &mut impl RaylibDraw, rec: Rectangle, wave_in: &[f32]) {
    let n = rec.width.trunc() as usize;
    let step = (wave_in.len() as f64 / n as f64).ceil() as usize;
    let wave: Vec<f32> = wave_in.iter().step_by(step).take(n).cloned().collect();
    let n_samples = wave.len();
    let max = wave
        .iter()
        .map(|x| x.abs())
        .reduce(f32::max)
        .unwrap_or_default();

    let spacing = (rec.width / (n_samples + 1) as f32);

    let center_y = (rec.y + rec.height / 2f32).trunc();

    let mut offset = spacing;

    let get_y = |sample: f32| (center_y - (sample / max) * (rec.height / 2.5f32)).trunc();
    let mut last_point = get_y(wave[0]);
    for sample in wave {
        let y = get_y(sample);
        const COLOR: Color = Color::BLUEVIOLET;
        d.draw_line_ex(
            Vector2::new(rec.x + offset - spacing, last_point),
            Vector2::new(rec.x + offset, y),
            1f32,
            COLOR,
        );
        last_point = y;
        // d.draw_pixel((rec.x + offset) as _, y as _, Color::RED);
        offset += spacing;
    }
}

pub fn draw_wave_box(d: &mut impl RaylibDraw, rec: Rectangle, wave_in: &[f32]) {
    // center line
    d.draw_line_ex(
        Vector2::new(0f32, (rec.height / 2f32).trunc()),
        Vector2::new(rec.width, (rec.height / 2f32).trunc()),
        1f32,
        Color::GRAY,
    );
    draw_wave(d, rec, wave_in);
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

pub struct AudioSink {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl AudioSink {
    pub fn try_default() -> anyhow::Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            stream,
            stream_handle,
            sink,
        })
    }
}

impl Block<Wave> for AudioSink {
    type Output = Wave;

    fn process(&mut self, input: Wave) -> Self::Output {
        let source =
            rodio::buffer::SamplesBuffer::new(1, dsp::SR as u32, input.clone()).repeat_infinite();

        // let total_duration = input.len() as f32 / dsp::SR as f32;
        // let current_sound_pos = (sink.get_pos().as_secs_f32() / total_duration) % total_duration;

        self.sink.append(source);
        self.sink.set_volume(0.2);
        self.sink.play();

        input
    }

    fn process_and_visualize(
        &mut self,
        input: Wave,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let out = self.process(input);
        let mut tx = context.get_texture(BOX_SIZE as _, BOX_SIZE as _);
        let mut d = context.rl.begin_drawing(context.thread);
        let mut d = d.begin_texture_mode(context.thread, &mut tx);

        let center = draw_simple_bock(&mut d, "Sink");

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
}

pub enum WaveView {
    Grow,
    Small,
}

impl Block<Wave> for WaveView {
    type Output = Wave;

    fn process(&mut self, input: Wave) -> Self::Output {
        input
    }

    fn process_and_visualize(
        &mut self,
        input: Wave,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let out = self.process(input);

        let unit = 2f32 * BOX_SIZE;
        let rec = match self {
            WaveView::Grow => {
                // unit per SR
                Rectangle {
                    width: ((out.len() / dsp::SR) + 1) as f32 * (unit * 2f32),
                    height: 50f32,
                    x: 0f32,
                    y: 0f32,
                }
            }
            WaveView::Small => {
                // fit everything in unit
                Rectangle {
                    width: unit,
                    height: 50f32,
                    x: 0f32,
                    y: 0f32,
                }
            }
        };

        let mut tx = context.get_texture(rec.width as _, rec.height as _);
        let mut d = context.rl.begin_drawing(context.thread);
        let mut d = d.begin_texture_mode(context.thread, &mut tx);

        draw_wave_box(&mut d, rec, &out);

        drop(d);
        (
            out,
            VisualizeResult::Block {
                texture: tx,
                input_connections: vec![Vector2::new(0f32, rec.height / 2f32)],
                output_connections: vec![Vector2::new(rec.width, rec.height / 2f32)],
            },
        )
    }
}
