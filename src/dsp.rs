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

    pub fn create_sinusoid(freq: f32, phase: f32, duration: std::time::Duration) -> Wave {
        let num_samples = (SR as f32 * duration.as_secs_f32()) as usize;

        let step = duration.as_secs_f32() / num_samples as f32;

        let mut wave = vec![0f32; num_samples];
        for i in 0..num_samples {
            let value = 2.0 * PI * freq * (step * i as f32) + phase;
            wave[i] = value.sin();
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

        #[derive(Clone)]
        pub enum WaveType {
            Sinusoid,
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
                    WaveType::Sinusoid => {
                        signals::create_sinusoid(controls.freq, controls.phase, controls.duration)
                    }
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

                let h = (vis::BOX_SIZE / 2f32) + vis::T;
                // vis::draw_wave(
                //     &mut d,
                //     Rectangle {
                //         x: vis::T,
                //         y: 0f32,
                //         width: vis::BOX_SIZE,
                //         height: vis::BOX_SIZE,
                //     },
                //     &out[..SR],
                // );
                d.draw_text(
                    &format!("{}hz", controls.freq),
                    (vis::T + 2f32) as _,
                    (h + 2f32) as _,
                    1,
                    vis::TEXT_COLOR,
                );
                vis::draw_border(
                    &mut d,
                    Rectangle {
                        height: vis::BOX_SIZE,
                        width: vis::BOX_SIZE,
                        ..Default::default()
                    },
                );
                drop(d);
                (out, VisualizeResult::SimpleTexture(tx))
            }
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

            let h = (vis::BOX_SIZE / 2f32) + vis::T;
            d.draw_text(
                &format!("{:?}", self),
                (vis::T + 2f32) as _,
                (h + 2f32) as _,
                1,
                vis::TEXT_COLOR,
            );
            vis::draw_border(
                &mut d,
                Rectangle {
                    height: vis::BOX_SIZE,
                    width: vis::BOX_SIZE,
                    ..Default::default()
                },
            );
            drop(d);
            (out, VisualizeResult::SimpleTexture(tx))
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

            let ((a_texture, a_inputs, a_outputs), (b_texture, b_inputs, b_output)) =
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
            db.draw_texture_rec(
                &a_texture,
                Rectangle {
                    width: a_texture.width() as _,
                    height: -a_texture.height() as _,
                    ..Default::default()
                },
                Vector2::new(0f32, 0f32),
                Color::WHITE,
            );
            db.draw_texture_rec(
                &b_texture,
                Rectangle {
                    width: b_texture.width() as _,
                    height: -b_texture.height() as _,
                    ..Default::default()
                },
                Vector2::new(0f32, (a_texture.height() as f32 + pad) as f32),
                Color::WHITE,
            );
            drop(db);
            drop(d);

            ((a, b), VisualizeResult::SimpleTexture(tx))
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

            let ((a_texture, a_inputs, a_outputs), (b_texture, b_inputs, b_output)) =
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

            let pad = vis::T * 40f32;
            let mut tx = context.get_texture((max_w as f32 + pad) as _, max_h as _);
            let mut d = context.rl.begin_drawing(context.thread);
            let mut d = d.begin_texture_mode(context.thread, &mut tx);
            let mut db = d.begin_blend_mode(BlendMode::BLEND_ALPHA);
            db.draw_texture_rec(
                &a_texture,
                Rectangle {
                    width: a_texture.width() as _,
                    height: -a_texture.height() as _,
                    ..Default::default()
                },
                Vector2::new(0f32, ((max_h as f32) - a_texture.height() as f32) / 2f32),
                Color::WHITE,
            );
            db.draw_texture_rec(
                &b_texture,
                Rectangle {
                    width: b_texture.width() as _,
                    height: -b_texture.height() as _,
                    ..Default::default()
                },
                Vector2::new(
                    (b_texture.width() as f32 + pad) as f32,
                    ((max_h as f32) - b_texture.height() as f32) / 2f32,
                ),
                Color::WHITE,
            );
            drop(db);
            drop(d);

            (out, VisualizeResult::SimpleTexture(tx))
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
