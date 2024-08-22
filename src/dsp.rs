use std::f32::consts::PI;
pub const SR: usize = 44100;

pub type Wave = Vec<f32>;

pub trait Block {
    type Input;
    type Output;

    fn process(&mut self, input: Self::Input) -> Self::Output;
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

    pub enum Basic<const N: usize>  {
        Mix,
    }

    impl<const N: usize> Block for Basic<N> {
        type Input = [Wave; N];
        type Output = Wave;
    
        fn process(&mut self, input: Self::Input) -> Self::Output {
            match self {
                Basic::Mix => {
                    let len = input[0].len();
                    assert!(input.iter().all(|wave| wave.len() == len), "All input waves must have the same length");
    
                    let mut wave = vec![0f32; len];
                    for i in 0..len {
                        let sum: f32 = input.iter().map(|w| w[i]).sum();
                        wave[i] = sum / N as f32;
                    }
    
                    wave
                }
            }
        }
    }

    pub trait Join {
        fn connect<I1, OC, O2, S2: Block<Input = OC, Output = O2>>(
            self,
            other: S2,
        ) -> impl Block<Input = I1, Output = O2>
        where
            Self: Block<Input = I1, Output = OC>;
    }

    pub struct Channels<const N: usize>;

    impl<const N: usize> Block for Channels<N> {
        type Input = [Wave; N];

        type Output = [Wave; N];

        fn process(&mut self, x: Self::Input) -> Self::Output {
            x
        }
    }

    pub struct JoinedBlocks<S1, S2> {
        input: S1,
        output: S2,
    }

    impl<S1, S2, I1, OC, O2> Block for JoinedBlocks<S1, S2>
    where
        S1: Block<Input = I1, Output = OC>,
        S2: Block<Input = OC, Output = O2>,
    {
        type Input = I1;

        type Output = O2;

        fn process(&mut self, input: Self::Input) -> Self::Output {
            let x = self.input.process(input);
            self.output.process(x)
        }
    }

    impl<T> Join for T {
        fn connect<I1, OC, O2, S2: Block<Input = OC, Output = O2>>(
            self,
            other: S2,
        ) -> impl Block<Input = I1, Output = O2>
        where
            Self: Block<Input = I1, Output = OC>,
        {
            JoinedBlocks {
                input: self,
                output: other,
            }
        }
    }
}
