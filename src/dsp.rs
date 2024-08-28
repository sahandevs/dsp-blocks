use crate::vis;
use crate::vis::{DrawContext, VisualizeResult};
use raylib::texture::RaylibTexture2D;
use raylib::{color::Color, ffi::BlendMode, math::Vector2, prelude::RaylibBlendModeExt};
use raylib::{
    math::Rectangle,
    prelude::{RaylibDraw, RaylibTextureModeExt},
};
use std::f32::consts::PI;
pub const SR: usize = 44100;

pub type Wave = Vec<f32>;

pub trait Block<Input> {
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
}

pub mod signals {
    pub use super::*;

    pub fn create_periodic_wave<T: Fn(f32) -> f32>(duration: std::time::Duration, fun: T) -> Wave {
        let num_samples = (SR as f32 * duration.as_secs_f32()) as usize;

        let step = duration.as_secs_f32() / num_samples as f32;

        let mut wave = vec![0f32; num_samples];
        for i in 0..num_samples {
            wave[i] = fun(step * i as f32);
        }
        wave
    }
}

pub mod blocks {

    pub use super::*;

    pub mod synths {

        use crate::vis;

        pub use super::*;

        pub struct Oscillator;

        #[derive(Clone, Debug)]
        pub enum WaveType {
            Sinusoid,
            Square,
            Triangle,
            Sawtooth,
        }

        #[derive(Clone)]
        pub struct OscillatorControls {
            pub freq: f32,
            pub phase: f32,
            pub duration: std::time::Duration,
            pub wave: WaveType,
        }

        impl Block<OscillatorControls> for Oscillator {
            type Output = Wave;

            fn process(&mut self, controls: OscillatorControls) -> Self::Output {
                match controls.wave {
                    WaveType::Sinusoid => signals::create_periodic_wave(controls.duration, |x| {
                        (2.0 * PI * controls.freq * (x as f32) + controls.phase).sin()
                    }),
                    WaveType::Square => signals::create_periodic_wave(controls.duration, |x| {
                        (2.0 * PI * controls.freq * (x as f32) + controls.phase)
                            .sin()
                            .signum()
                    }),
                    WaveType::Triangle => signals::create_periodic_wave(controls.duration, |x| {
                        (2.0 / PI)
                            * (2.0 * PI * controls.freq * (x as f32) + controls.phase)
                                .sin()
                                .asin()
                    }),
                    WaveType::Sawtooth => signals::create_periodic_wave(controls.duration, |x| {
                        let phase_offset = controls.phase / (2.0 * PI);
                        2.0 * (((x * controls.freq + phase_offset) % 1.0) - 0.5)
                    }),
                }
            }

            fn process_and_visualize(
                &mut self,
                controls: OscillatorControls,
                context: &mut DrawContext,
            ) -> (Self::Output, VisualizeResult) {
                let out = self.process(controls.clone());
                let mut tx = context.get_texture(vis::BOX_SIZE as _, vis::BOX_SIZE as _);
                let mut d = context.rl.begin_drawing(context.thread);
                let mut d = d.begin_texture_mode(context.thread, &mut tx);

                let center = vis::draw_simple_bock(
                    &mut d,
                    &format!("{:?}\n{}hz", controls.wave, controls.freq),
                );
                drop(d);
                (
                    out,
                    VisualizeResult::Block {
                        texture: tx,
                        input_connections: vec![Vector2::new(0f32, center.y)],
                        output_connections: vec![Vector2::new(vis::BOX_SIZE, center.y)],
                    },
                )
            }
        }
    }

    pub struct Discard;
    impl<T> Block<T> for Discard {
        type Output = ();

        fn process(&mut self, _input: T) -> Self::Output {
            ()
        }
    }

    #[derive(Debug)]
    pub enum Basic<const N: usize> {
        Mix,
    }

    impl<T: IntoArray<[Wave; N]>, const N: usize> Block<T> for Basic<N> {
        type Output = Wave;

        fn process(&mut self, input: T) -> Self::Output {
            let input = input.into();
            match self {
                Basic::Mix => {
                    let len = input[0].len();
                    assert!(
                        input.iter().all(|wave| wave.len() == len),
                        "All input waves must have the same length"
                    );

                    let mut wave = vec![0f32; len];
                    for i in 0..len {
                        let sum: f32 = input.iter().map(|w| w[i]).sum();
                        wave[i] = sum / N as f32;
                    }

                    wave
                }
            }
        }

        fn process_and_visualize(
            &mut self,
            input: T,
            context: &mut DrawContext,
        ) -> (Self::Output, VisualizeResult) {
            let out = self.process(input);
            let mut tx = context.get_texture(vis::BOX_SIZE as _, vis::BOX_SIZE as _);
            let mut d = context.rl.begin_drawing(context.thread);
            let mut d = d.begin_texture_mode(context.thread, &mut tx);
            let center = vis::draw_simple_bock(&mut d, &format!("{:?}", self));
            drop(d);
            (
                out,
                VisualizeResult::Block {
                    texture: tx,
                    input_connections: vec![Vector2::new(0f32, center.y)].repeat(N),
                    output_connections: vec![Vector2::new(vis::BOX_SIZE, center.y)],
                },
            )
        }
    }

    pub trait CanJoin {
        fn join<I1, I2, O1, O2, S: Block<I2, Output = O2>>(
            self,
            other: S,
        ) -> impl Block<(I1, I2), Output = (O1, O2)>
        where
            Self: Block<I1, Output = O1>;
    }

    pub struct JoinedBlocks<S1, S2> {
        a: S1,
        b: S2,
    }

    impl<S1, S2, I1, I2, O1, O2> Block<(I1, I2)> for JoinedBlocks<S1, S2>
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
                        return ((a, b), VisualizeResult::None)
                    }
                    (VisualizeResult::None, VisualizeResult::SimpleTexture(x))
                    | (VisualizeResult::SimpleTexture(x), VisualizeResult::None) => {
                        return ((a, b), VisualizeResult::SimpleTexture(x))
                    }
                    (
                        VisualizeResult::None,
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                    )
                    | (
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                        VisualizeResult::None,
                    ) => {
                        return (
                            (a, b),
                            VisualizeResult::Block {
                                texture,
                                input_connections,
                                output_connections,
                            },
                        )
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

            let b_offset = (max_w as f32) - b_texture.width() as f32;
            db.draw_texture_rec(
                &b_texture,
                Rectangle {
                    width: b_texture.width() as _,
                    height: -b_texture.height() as _,
                    ..Default::default()
                },
                Vector2::new(b_offset, (a_texture.height() as f32 + pad) as f32),
                Color::WHITE,
            );
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
    }

    impl<T> CanJoin for T {
        fn join<I1, I2, O1, O2, S: Block<I2, Output = O2>>(
            self,
            other: S,
        ) -> impl Block<(I1, I2), Output = (O1, O2)>
        where
            Self: Block<I1, Output = O1>,
        {
            JoinedBlocks { a: self, b: other }
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

    pub struct ConnectedBlocks<S1, S2> {
        input: S1,
        output: S2,
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
                        return ((out), VisualizeResult::None)
                    }
                    (VisualizeResult::None, VisualizeResult::SimpleTexture(x))
                    | (VisualizeResult::SimpleTexture(x), VisualizeResult::None) => {
                        return ((out), VisualizeResult::SimpleTexture(x))
                    }
                    (
                        VisualizeResult::None,
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                    )
                    | (
                        VisualizeResult::Block {
                            texture,
                            input_connections,
                            output_connections,
                        },
                        VisualizeResult::None,
                    ) => {
                        return (
                            (out),
                            VisualizeResult::Block {
                                texture,
                                input_connections,
                                output_connections,
                            },
                        )
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
            }
        }
    }
}
