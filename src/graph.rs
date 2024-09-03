use crate::vis::{DrawContext, VisualizeResult};
use crate::{control, vis};
use raylib::texture::RaylibTexture2D;
use raylib::{color::Color, ffi::BlendMode, math::Vector2, prelude::RaylibBlendModeExt};
use raylib::{
    math::Rectangle,
    prelude::{RaylibDraw, RaylibTextureModeExt},
};
use std::fmt::Debug;

#[derive(Debug)]
pub struct Discard;
impl<T> Block<T> for Discard {
    type Output = ();

    fn process(&mut self, _input: T) -> Self::Output {
        ()
    }
}

pub trait Block<Input>: Debug {
    type Output;

    fn process(&mut self, input: Input) -> Self::Output;

    fn process_and_visualize(
        &mut self,
        input: Input,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let _ = context;
        let x = self.process(input);
        (x, VisualizeResult::None)
    }

    fn on_hover(
        &mut self,
        pos: Vector2,
        context: &mut control::ControlContext,
    ) -> control::ControlResult {
        let _ = context;
        let _ = pos;
        control::ControlResult::Passthrough
    }

    fn on_unhover(&mut self, context: &mut control::ControlContext) -> control::ControlResult {
        let _ = context;
        control::ControlResult::Passthrough
    }
}

pub trait CanStack {
    fn join<I1, I2, O1, O2, S: Block<I2, Output = O2>>(
        self,
        other: S,
    ) -> impl Block<(I1, I2), Output = (O1, O2)>
    where
        Self: Block<I1, Output = O1>;
}

#[derive(Debug)]
pub struct StackedBlocks<S1, S2> {
    a: S1,
    b: S2,

    a_tx_rec: Rectangle,
    b_tx_rec: Rectangle,
}

impl<S1, S2, I1, I2, O1, O2> Block<(I1, I2)> for StackedBlocks<S1, S2>
where
    S1: Block<I1, Output = O1>,
    S2: Block<I2, Output = O2>,
{
    type Output = (O1, O2);

    fn process(&mut self, input: (I1, I2)) -> Self::Output {
        let a = self.a.process(input.0);
        let b = self.b.process(input.1);
        (a, b)
    }

    fn process_and_visualize(
        &mut self,
        input: (I1, I2),
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let (a, a_txt) = self.a.process_and_visualize(input.0, context);
        let (b, b_txt) = self.b.process_and_visualize(input.1, context);

        let ((a_texture, mut a_inputs, mut a_outputs), (b_texture, mut b_inputs, mut b_output)) =
            match (a_txt, b_txt) {
                (VisualizeResult::None, VisualizeResult::None) => {
                    self.a_tx_rec = Default::default();
                    self.b_tx_rec = Default::default();
                    return (((a, b)), VisualizeResult::None);
                }
                (VisualizeResult::None, VisualizeResult::SimpleTexture(x)) => {
                    self.a_tx_rec = Default::default();
                    self.b_tx_rec.width = x.width().abs() as _;
                    self.b_tx_rec.height = x.height().abs() as _;
                    self.b_tx_rec.x = 0f32;
                    self.b_tx_rec.y = 0f32;
                    return (((a, b)), VisualizeResult::SimpleTexture(x));
                }
                (VisualizeResult::SimpleTexture(x), VisualizeResult::None) => {
                    self.b_tx_rec = Default::default();
                    self.a_tx_rec.width = x.width().abs() as _;
                    self.a_tx_rec.height = x.height().abs() as _;
                    self.a_tx_rec.x = 0f32;
                    self.a_tx_rec.y = 0f32;
                    return (((a, b)), VisualizeResult::SimpleTexture(x));
                }
                (
                    VisualizeResult::None,
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                ) => {
                    self.a_tx_rec = Default::default();
                    self.b_tx_rec.width = texture.width().abs() as _;
                    self.b_tx_rec.height = texture.height().abs() as _;
                    self.b_tx_rec.x = 0f32;
                    self.b_tx_rec.y = 0f32;
                    return (
                        ((a, b)),
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                    );
                }
                (
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                    VisualizeResult::None,
                ) => {
                    self.b_tx_rec = Default::default();
                    self.a_tx_rec.width = texture.width().abs() as _;
                    self.a_tx_rec.height = texture.height().abs() as _;
                    self.a_tx_rec.x = 0f32;
                    self.a_tx_rec.y = 0f32;
                    return (
                        ((a, b)),
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                    );
                }
                (VisualizeResult::SimpleTexture(a), VisualizeResult::SimpleTexture(b)) => {
                    ((a, vec![], vec![]), (b, vec![], vec![]))
                }
                (
                    VisualizeResult::SimpleTexture(a),
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                ) => (
                    (a, vec![], vec![]),
                    (texture, input_connections, output_connections),
                ),
                (
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                    VisualizeResult::SimpleTexture(b),
                ) => (
                    (texture, input_connections, output_connections),
                    (b, vec![], vec![]),
                ),
                (
                    VisualizeResult::Block {
                        texture: a,
                        input_connections: a_input,
                        output_connections: a_output,
                    },
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                ) => (
                    (a, a_input, a_output),
                    (texture, input_connections, output_connections),
                ),
            };

        let max_w = a_texture.width().max(b_texture.width());
        let max_h = a_texture.height() + b_texture.height();

        let pad = vis::T * 3f32;
        let mut tx = context.get_texture(max_w as _, (max_h as f32 + pad) as _);
        tx.set_texture_filter(
            context.thread,
            raylib::ffi::TextureFilter::TEXTURE_FILTER_ANISOTROPIC_16X,
        );
        let mut d = context.rl.begin_drawing(context.thread);
        let mut d = d.begin_texture_mode(context.thread, &mut tx);
        let mut db = d.begin_blend_mode(BlendMode::BLEND_ALPHA);

        let a_offset = (max_w as f32) - a_texture.width() as f32;
        db.draw_texture_rec(
            &a_texture,
            Rectangle {
                width: a_texture.width() as _,
                height: -a_texture.height() as _,
                ..Default::default()
            },
            Vector2::new(a_offset, 0f32),
            Color::WHITE,
        );
        self.a_tx_rec.width = a_texture.width() as _;
        self.a_tx_rec.height = a_texture.height() as _;
        self.a_tx_rec.x = a_offset;
        self.a_tx_rec.y = 0f32;

        let b_offset = (max_w as f32) - b_texture.width() as f32;
        let b_offset_y = (a_texture.height() as f32 + pad) as f32;
        db.draw_texture_rec(
            &b_texture,
            Rectangle {
                width: b_texture.width() as _,
                height: -b_texture.height() as _,
                ..Default::default()
            },
            Vector2::new(b_offset, b_offset_y),
            Color::WHITE,
        );
        self.b_tx_rec.width = b_texture.width() as _;
        self.b_tx_rec.height = b_texture.height() as _;
        self.b_tx_rec.x = b_offset;
        self.b_tx_rec.y = b_offset_y;
        drop(db);
        drop(d);

        for b in b_inputs.iter_mut() {
            b.y += a_texture.height() as f32 + pad;
            b.x += b_offset;
        }
        for b in b_output.iter_mut() {
            b.y += a_texture.height() as f32 + pad;
            b.x += b_offset;
        }
        for a in a_inputs.iter_mut() {
            a.x += a_offset;
        }
        for a in a_outputs.iter_mut() {
            a.x += a_offset;
        }
        a_inputs.extend(b_inputs);
        a_outputs.extend(b_output);

        (
            (a, b),
            VisualizeResult::Block {
                texture: tx,
                input_connections: a_inputs,
                output_connections: a_outputs,
            },
        )
    }

    fn on_hover(
        &mut self,
        pos: Vector2,
        context: &mut control::ControlContext,
    ) -> control::ControlResult {
        if self.a_tx_rec.check_collision_point_rec(pos) {
            self.a.on_hover(
                pos - Vector2::new(self.a_tx_rec.x, self.a_tx_rec.y),
                context,
            );
            self.b.on_unhover(context);
        }

        if self.b_tx_rec.check_collision_point_rec(pos) {
            self.b.on_hover(
                pos - Vector2::new(self.b_tx_rec.x, self.b_tx_rec.y),
                context,
            );
            self.a.on_unhover(context);
        }

        control::ControlResult::Passthrough
    }

    fn on_unhover(&mut self, context: &mut control::ControlContext) -> control::ControlResult {
        self.a.on_unhover(context);
        self.b.on_unhover(context);
        control::ControlResult::Passthrough
    }
}

impl<T> CanStack for T {
    fn join<I1, I2, O1, O2, S: Block<I2, Output = O2>>(
        self,
        other: S,
    ) -> impl Block<(I1, I2), Output = (O1, O2)>
    where
        Self: Block<I1, Output = O1>,
    {
        StackedBlocks {
            a: self,
            b: other,
            b_tx_rec: Default::default(),
            a_tx_rec: Default::default(),
        }
    }
}

pub trait IntoArray<T>: Sized {
    fn into(self) -> T;
}

impl<T> IntoArray<[T; 2]> for (T, T) {
    fn into(self) -> [T; 2] {
        [self.0, self.1]
    }
}
impl<T> IntoArray<[T; 3]> for ((T, T), T) {
    fn into(self) -> [T; 3] {
        let ((a, b), c) = self;
        [a, b, c]
    }
}
impl<T> IntoArray<[T; 4]> for (((T, T), T), T) {
    fn into(self) -> [T; 4] {
        let (((a, b), c), d) = self;
        [a, b, c, d]
    }
}

pub trait CanConnect {
    fn connect<I1, OC, O2, S2: Block<OC, Output = O2>>(
        self,
        other: S2,
    ) -> impl Block<I1, Output = O2>
    where
        Self: Block<I1, Output = OC>;
}

#[derive(Debug)]
pub struct ConnectedBlocks<S1, S2> {
    input: S1,
    output: S2,

    in_tx_rec: Rectangle,
    out_tx_rec: Rectangle,
}

impl<S1, S2, I1, OC, O2> Block<I1> for ConnectedBlocks<S1, S2>
where
    S1: Block<I1, Output = OC>,
    S2: Block<OC, Output = O2>,
{
    type Output = O2;

    fn process(&mut self, input: I1) -> Self::Output {
        let x = self.input.process(input);
        self.output.process(x)
    }

    fn process_and_visualize(
        &mut self,
        input: I1,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let (x, in_txt) = self.input.process_and_visualize(input, context);
        let (out, out_txt) = self.output.process_and_visualize(x, context);

        let ((a_texture, mut a_inputs, mut a_outputs), (b_texture, mut b_inputs, mut b_output)) =
            match (in_txt, out_txt) {
                (VisualizeResult::None, VisualizeResult::None) => {
                    self.in_tx_rec = Default::default();
                    self.out_tx_rec = Default::default();
                    return ((out), VisualizeResult::None);
                }
                (VisualizeResult::None, VisualizeResult::SimpleTexture(x)) => {
                    self.in_tx_rec = Default::default();
                    self.out_tx_rec.width = x.width().abs() as _;
                    self.out_tx_rec.height = x.height().abs() as _;
                    self.out_tx_rec.x = 0f32;
                    self.out_tx_rec.y = 0f32;
                    return ((out), VisualizeResult::SimpleTexture(x));
                }
                (VisualizeResult::SimpleTexture(x), VisualizeResult::None) => {
                    self.out_tx_rec = Default::default();
                    self.in_tx_rec.width = x.width().abs() as _;
                    self.in_tx_rec.height = x.height().abs() as _;
                    self.in_tx_rec.x = 0f32;
                    self.in_tx_rec.y = 0f32;
                    return ((out), VisualizeResult::SimpleTexture(x));
                }
                (
                    VisualizeResult::None,
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                ) => {
                    self.in_tx_rec = Default::default();
                    self.out_tx_rec.width = texture.width().abs() as _;
                    self.out_tx_rec.height = texture.height().abs() as _;
                    self.out_tx_rec.x = 0f32;
                    self.out_tx_rec.y = 0f32;
                    return (
                        (out),
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                    );
                }
                (
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                    VisualizeResult::None,
                ) => {
                    self.out_tx_rec = Default::default();
                    self.in_tx_rec.width = texture.width().abs() as _;
                    self.in_tx_rec.height = texture.height().abs() as _;
                    self.in_tx_rec.x = 0f32;
                    self.in_tx_rec.y = 0f32;
                    return (
                        (out),
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                    );
                }
                (VisualizeResult::SimpleTexture(a), VisualizeResult::SimpleTexture(b)) => {
                    ((a, vec![], vec![]), (b, vec![], vec![]))
                }
                (
                    VisualizeResult::SimpleTexture(a),
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                ) => (
                    (a, vec![], vec![]),
                    (texture, input_connections, output_connections),
                ),
                (
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                    VisualizeResult::SimpleTexture(b),
                ) => (
                    (texture, input_connections, output_connections),
                    (b, vec![], vec![]),
                ),
                (
                    VisualizeResult::Block {
                        texture: a,
                        input_connections: a_input,
                        output_connections: a_output,
                    },
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    },
                ) => (
                    (a, a_input, a_output),
                    (texture, input_connections, output_connections),
                ),
            };

        let max_w = a_texture.width() + b_texture.width();
        let max_h = a_texture.height().max(b_texture.height());

        let pad = vis::T * 10f32;
        let mut tx = context.get_texture((max_w as f32 + pad) as _, max_h as _);
        tx.set_texture_filter(
            context.thread,
            raylib::ffi::TextureFilter::TEXTURE_FILTER_ANISOTROPIC_16X,
        );
        let mut d = context.rl.begin_drawing(context.thread);
        let mut d = d.begin_texture_mode(context.thread, &mut tx);
        let mut db = d.begin_blend_mode(BlendMode::BLEND_ALPHA);

        let a_offset = (((max_h as f32) - a_texture.height() as f32) / 2f32).trunc();
        db.draw_texture_rec(
            &a_texture,
            Rectangle {
                width: a_texture.width() as _,
                height: -a_texture.height() as _,
                ..Default::default()
            },
            Vector2::new(0f32, a_offset),
            Color::WHITE,
        );
        self.in_tx_rec.width = a_texture.width() as _;
        self.in_tx_rec.height = a_texture.height() as _;
        self.in_tx_rec.x = 0f32;
        self.in_tx_rec.y = a_offset;
        let b_txt_pos = Vector2::new(
            (a_texture.width() as f32 + pad) as f32,
            (((max_h as f32) - b_texture.height() as f32) / 2f32).trunc(),
        );
        db.draw_texture_rec(
            &b_texture,
            Rectangle {
                width: b_texture.width() as _,
                height: -b_texture.height() as _,
                ..Default::default()
            },
            b_txt_pos,
            Color::WHITE,
        );
        self.out_tx_rec.width = b_texture.width() as _;
        self.out_tx_rec.height = b_texture.height() as _;
        self.out_tx_rec.x = b_txt_pos.x;
        self.out_tx_rec.y = b_txt_pos.y;

        // draw connections
        assert_eq!(a_outputs.len(), b_inputs.len());
        for a in a_outputs.iter_mut() {
            a.y += a_offset;
        }

        for b in b_inputs.iter_mut() {
            *b += b_txt_pos;
        }

        for (a, b) in a_outputs.iter().zip(b_inputs.iter()) {
            db.draw_line_bezier(a, b, 1f32, vis::BORDER_COLOR);
        }

        drop(db);
        drop(d);

        for a in a_inputs.iter_mut() {
            a.y += a_offset;
        }

        for b in b_output.iter_mut() {
            *b += b_txt_pos;
        }

        (
            out,
            VisualizeResult::Block {
                texture: tx,
                input_connections: a_inputs,
                output_connections: b_output,
            },
        )
    }

    fn on_hover(
        &mut self,
        pos: Vector2,
        context: &mut control::ControlContext,
    ) -> control::ControlResult {
        if self.in_tx_rec.check_collision_point_rec(pos) {
            self.input.on_hover(
                pos - Vector2::new(self.in_tx_rec.x, self.in_tx_rec.y),
                context,
            );
            self.output.on_unhover(context);
        }

        if self.out_tx_rec.check_collision_point_rec(pos) {
            self.output.on_hover(
                pos - Vector2::new(self.out_tx_rec.x, self.out_tx_rec.y),
                context,
            );
            self.input.on_unhover(context);
        }

        control::ControlResult::Passthrough
    }

    fn on_unhover(&mut self, context: &mut control::ControlContext) -> control::ControlResult {
        self.input.on_unhover(context);
        self.output.on_unhover(context);
        control::ControlResult::Passthrough
    }
}

impl<T> CanConnect for T {
    fn connect<I1, OC, O2, S2: Block<OC, Output = O2>>(
        self,
        other: S2,
    ) -> impl Block<I1, Output = O2>
    where
        Self: Block<I1, Output = OC>,
    {
        ConnectedBlocks {
            input: self,
            output: other,
            in_tx_rec: Default::default(),
            out_tx_rec: Default::default(),
        }
    }
}