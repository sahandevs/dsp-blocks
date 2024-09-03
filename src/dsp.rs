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
        for i in 0..num_samples {
            wave[i] = fun(step * i as f32);
        }
        wave
    }
}

pub mod blocks {

    use crate::{
        graph::{Block, DInto},
        vis,
    };

    pub use super::*;

    pub mod synths {

        use crate::{graph::Block, vis};

        pub use super::*;

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
                vis::visualize_simple_box(
                    context,
                    &format!("{:?}\n{}Hz", controls.wave, controls.freq),
                    out,
                )
            }
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
                "All input waves must have the same length"
            );

            match self {
                Basic::Mix => {
                    let mut wave = vec![0f32; len];
                    for i in 0..len {
                        let sum: f32 = input.iter().map(|w| w[i]).sum();
                        wave[i] = sum / N as f32;
                    }

                    wave
                }
                Basic::Amp => {
                    let mut wave = vec![0f32; len];
                    for i in 0..len {
                        let amp: f32 = input.iter().map(|w| w[i]).fold(1f32, |acc, x| acc * x);
                        wave[i] = amp;
                    }

                    wave
                }
                Basic::Diff => {
                    let mut wave = vec![0f32; len];
                    for i in 1..len {
                        let acc: f32 = input.iter().map(|w| w[i]).sum();
                        wave[i] = acc;
                    }
                    for (i, v) in input[0].iter().enumerate() {
                        wave[i] = v - wave[i];
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
}
