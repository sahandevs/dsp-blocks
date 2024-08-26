use raylib::prelude::*;

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

    let center_y = rec.y + rec.height / 2f32;

    let mut offset = spacing;
    let l_w = (5f32 * (T * 1.5 / n_samples as f32)).max(1f32);

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
        offset += spacing;
    }
}
