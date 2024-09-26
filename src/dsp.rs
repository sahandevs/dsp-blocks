use crate::vis::{DrawContext, VisualizeResult};

use std::f32::consts::PI;
use std::fmt::Debug;
pub const SR: usize = 44100;

pub type Wave = Vec<f32>;

pub mod signals {
    pub use super::*;

    pub fn create_periodic_wave<T: Fn(f32) -> f32>(duration: std::time::Duration, fun: T) -> Wave {
        let num_samples = (SR as f32 * duration.as_secs_f32()) as usize;

        let step = duration.as_secs_f32() / num_samples as f32;

        let mut wave = vec![0f32; num_samples];
        for n in 0..num_samples {
            wave[n] = fun(step * n as f32);
        }
        wave
    }
}

pub mod blocks {

    use raylib::{
        math::Vector2,
        prelude::{RaylibDraw, RaylibTextureModeExt},
        texture::RaylibTexture2D,
    };

    use crate::{
        graph::{Block, DInto},
        vis,
    };

    pub use super::*;

    pub mod synths {

        use std::time::Duration;

        use crate::{graph::Block, vis};

        pub use super::*;

        /// https://tttapa.github.io/Pages/Mathematics/Systems-and-Control-Theory/Digital-filters/DTLTI-Systems,-Transfer-Functions,-and-the-Z-transform/Impulse-and-Step-Response.html#the-kronecker-delta-function
        #[derive(Debug)]
        pub enum KroneckerDelta {
            Start,
            Center,
            End,
        }

        impl Block<Duration> for KroneckerDelta {
            type Output = Wave;

            fn process(&mut self, duration: Duration) -> Self::Output {
                let num_samples = (SR as f32 * duration.as_secs_f32()) as usize;
                let mut wave = vec![0f32; num_samples];
                match self {
                    KroneckerDelta::Start => wave[0] = 1f32,
                    KroneckerDelta::Center => wave[num_samples / 2] = 1f32,
                    KroneckerDelta::End => wave[num_samples - 1] = 1f32,
                }
                wave
            }

            fn process_and_visualize(
                &mut self,
                dur: Duration,
                context: &mut DrawContext,
            ) -> (Self::Output, VisualizeResult) {
                let out = self.process(dur.clone());
                vis::visualize_simple_box(context, &format!("Delta\n{self:?}"), out)
            }
        }

        /// https://tttapa.github.io/Pages/Mathematics/Systems-and-Control-Theory/Digital-filters/DTLTI-Systems,-Transfer-Functions,-and-the-Z-transform/Impulse-and-Step-Response.html#the-heaviside-step-function
        #[derive(Debug)]
        pub struct HeavisideStep;

        impl Block<Duration> for HeavisideStep {
            type Output = Wave;

            fn process(&mut self, duration: Duration) -> Self::Output {
                let num_samples = (SR as f32 * duration.as_secs_f32()) as usize;
                let wave = vec![1f32; num_samples];
                wave
            }

            fn process_and_visualize(
                &mut self,
                dur: Duration,
                context: &mut DrawContext,
            ) -> (Self::Output, VisualizeResult) {
                let out = self.process(dur.clone());
                vis::visualize_simple_box(context, &format!("Step"), out)
            }
        }

        #[derive(Debug)]
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
                    WaveType::Sinusoid => signals::create_periodic_wave(controls.duration, |n| {
                        (2.0 * PI * controls.freq * (n as f32) + controls.phase).sin()
                    }),
                    WaveType::Square => signals::create_periodic_wave(controls.duration, |n| {
                        (2.0 * PI * controls.freq * (n as f32) + controls.phase)
                            .sin()
                            .signum()
                    }),
                    WaveType::Triangle => signals::create_periodic_wave(controls.duration, |n| {
                        (2.0 / PI)
                            * (2.0 * PI * controls.freq * (n as f32) + controls.phase)
                                .sin()
                                .asin()
                    }),
                    WaveType::Sawtooth => signals::create_periodic_wave(controls.duration, |n| {
                        let phase_offset = controls.phase / (2.0 * PI);
                        2.0 * (((n * controls.freq + phase_offset) % 1.0) - 0.5)
                    }),
                }
            }

            fn process_and_visualize(
                &mut self,
                controls: OscillatorControls,
                context: &mut DrawContext,
            ) -> (Self::Output, VisualizeResult) {
                let out = self.process(controls.clone());
                vis::visualize_simple_box(
                    context,
                    &format!("{:?}\n{}Hz", controls.wave, controls.freq),
                    out,
                )
            }
        }
    }

    #[derive(Debug)]
    pub enum AutoPad<const N: usize> {
        Start,
        End,
    }

    impl<I: DInto<[Wave; N]>, const N: usize> Block<I> for AutoPad<N> {
        type Output = I;

        fn process(&mut self, input: I) -> Self::Output {
            let mut waves: [Wave; N] = input.into();

            let max_len = waves.iter().map(|x| x.len()).max().unwrap_or_default();

            let mut out: [Wave; N] = core::array::from_fn(|_| Vec::with_capacity(0));
            for n in 0..N {
                let wil = waves[n].len();
                if wil < max_len {
                    match self {
                        AutoPad::Start => {
                            let mut w = vec![0f32; max_len];
                            w[max_len - wil..].copy_from_slice(&waves[n]);
                            waves[n] = w;
                        }
                        AutoPad::End => {
                            waves[n].extend([0f32].repeat(max_len - wil));
                        }
                    }
                }
                std::mem::swap(&mut out[n], &mut waves[n]);
            }

            DInto::from(out)
        }
        fn process_and_visualize(
            &mut self,
            input: I,
            context: &mut DrawContext,
        ) -> (Self::Output, VisualizeResult) {
            let out = self.process(input);
            vis::visualize_simple_box(context, &format!("Autopad\n{self:?}"), out)
        }
    }

    #[derive(Debug)]
    pub enum Basic<const N: usize> {
        Mix,
        Amp,
        Diff,
    }

    impl<T: DInto<[Wave; N]>, const N: usize> Block<T> for Basic<N> {
        type Output = Wave;

        fn process(&mut self, input: T) -> Self::Output {
            let input = input.into();
            let len = input[0].len();
            assert!(
                input.iter().all(|wave| wave.len() == len),
                "All input waves must have the same length {:?}",
                input.iter().map(|x| x.len()).collect::<Vec<_>>()
            );

            match self {
                Basic::Mix => {
                    let mut wave = vec![0f32; len];
                    for n in 0..len {
                        let sum: f32 = input.iter().map(|w| w[n]).sum();
                        wave[n] = sum / N as f32;
                    }

                    wave
                }
                Basic::Amp => {
                    let mut wave = vec![0f32; len];
                    for n in 0..len {
                        let amp: f32 = input.iter().map(|w| w[n]).fold(1f32, |acc, x| acc * x);
                        wave[n] = amp;
                    }

                    wave
                }
                Basic::Diff => {
                    let mut wave = vec![0f32; len];
                    for n in 1..len {
                        let acc: f32 = input.iter().map(|w| w[n]).sum();
                        wave[n] = acc;
                    }
                    for (n, v) in input[0].iter().enumerate() {
                        wave[n] = v - wave[n];
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
            let (out, vis_result) = vis::visualize_simple_box(context, &format!("{:?}", self), out);

            (
                out,
                match vis_result {
                    VisualizeResult::Block {
                        texture,
                        input_connections,
                        output_connections,
                    } => VisualizeResult::Block {
                        texture,
                        input_connections: input_connections.repeat(N),
                        output_connections,
                    },
                    x => x,
                },
            )
        }
    }

    #[derive(Debug, tidy_builder::Builder)]
    pub struct WindowSetting {
        #[builder(value = 1024)]
        frame_size: usize,
        #[builder(value = 512)]
        hop_length: usize,
    }

    impl Default for WindowSetting {
        fn default() -> Self {
            Self::builder().build()
        }
    }

    #[derive(Debug, Default)]
    pub enum EnvelopeType {
        #[default]
        Amp,
    }

    #[derive(Debug, tidy_builder::Builder)]
    pub struct EnvelopeBlock {
        #[builder(value = default)]
        pub t: EnvelopeType,
        #[builder(value = default)]
        pub window: WindowSetting,
    }

    impl Default for EnvelopeBlock {
        fn default() -> Self {
            Self::builder().build()
        }
    }

    impl Block<Wave> for EnvelopeBlock {
        type Output = Wave;

        fn process(&mut self, input: Wave) -> Self::Output {
            // shape of output depands on the hop_length
            let mut out =
                vec![0f32; (input.len() as f32 / self.window.hop_length as f32).ceil() as usize];

            let mut w_start = 0;
            for slot in out.iter_mut() {
                let frame =
                    &input[w_start..(self.window.frame_size + w_start).min(input.len() - 1)];
                match self.t {
                    EnvelopeType::Amp => {
                        *slot = frame.iter().copied().reduce(f32::max).unwrap_or_default();
                    }
                }
                w_start += self.window.hop_length;
            }
            out
        }

        fn process_and_visualize(
            &mut self,
            input: Wave,
            context: &mut DrawContext,
        ) -> (Self::Output, VisualizeResult) {
            let out = self.process(input);
            vis::visualize_simple_box(
                context,
                &format!(
                    "Envelope\n{:?}\n{},{}",
                    self.t, self.window.frame_size, self.window.hop_length
                ),
                out,
            )
        }
    }

    #[derive(Debug, Default)]
    pub struct ConstMultiplier(pub f32);

    impl Block<Wave> for ConstMultiplier {
        type Output = Wave;

        fn process(&mut self, mut input: Wave) -> Self::Output {
            for sample in input.iter_mut() {
                *sample *= self.0;
            }
            input
        }

        fn process_and_visualize(
            &mut self,
            input: Wave,
            context: &mut DrawContext,
        ) -> (Self::Output, VisualizeResult) {
            let out = self.process(input);
            let mut tx = context.get_texture(vis::BOX_SIZE as _, vis::BOX_SIZE as _);
            tx.set_texture_filter(
                context.thread,
                raylib::ffi::TextureFilter::TEXTURE_FILTER_ANISOTROPIC_16X,
            );
            let mut d = context.rl.begin_drawing(context.thread);
            let mut d = d.begin_texture_mode(context.thread, &mut tx);
            let pad = vis::T * 3f32;

            let center = ((vis::BOX_SIZE - pad * 2f32) / 2f32) + pad;
            d.draw_triangle_lines(
                Vector2::new(0f32 + pad, 0f32 + pad),
                Vector2::new(0f32 + pad, vis::BOX_SIZE - pad),
                Vector2::new(vis::BOX_SIZE - pad, center),
                vis::BORDER_COLOR,
            );
            d.draw_text(
                &format!("* {}", self.0),
                center as _,
                (pad as i32) / 2,
                1,
                vis::TEXT_COLOR,
            );
            drop(d);
            (
                out,
                VisualizeResult::Block {
                    texture: tx,
                    input_connections: vec![Vector2::new(0f32 + pad, center)],
                    output_connections: vec![Vector2::new(vis::BOX_SIZE - pad, center)],
                },
            )
        }
    }
}
