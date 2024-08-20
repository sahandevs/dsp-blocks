use std::f32::consts::PI;
pub const SR: usize = 44100;

pub type Wave = Vec<f32>;

pub trait System {
    type Input;
    type Output;

    fn process(&self, input: Self::Input) -> Self::Output;
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

pub mod systems {

    pub use super::*;

    pub enum BinaryOpSystem {
        Mix,
    }

    impl System for BinaryOpSystem {
        type Input = (Wave, Wave);

        type Output = Wave;

        fn process(&self, (a, b): Self::Input) -> Self::Output {
            match self {
                BinaryOpSystem::Mix => {
                    assert!(a.len() == b.len(), "{} {}", a.len(), b.len());

                    let mut wave = vec![0f32; a.len()];
                    for (i, (a, b)) in a.iter().zip(b.iter()).enumerate() {
                        wave[i] = (a + b) / 2f32;
                    }

                    wave
                }
            }
        }
    }

    pub trait ConnectedSystems {
        fn connect<I1, OC, O2, S2: System<Input = OC, Output = O2>>(
            self,
            other: S2,
        ) -> impl System<Input = I1, Output = O2>
        where
            Self: System<Input = I1, Output = OC>;
    }

    pub struct SterioSystem;

    impl System for SterioSystem {
        type Input = (Wave, Wave);

        type Output = (Wave, Wave);

        fn process(&self, (a, b): Self::Input) -> Self::Output {
            (a, b)
        }
    }

    pub struct ConnectedSystem<S1, S2> {
        input: S1,
        output: S2,
    }

    impl<S1, S2, I1, OC, O2> System for ConnectedSystem<S1, S2>
    where
        S1: System<Input = I1, Output = OC>,
        S2: System<Input = OC, Output = O2>,
    {
        type Input = I1;

        type Output = O2;

        fn process(&self, input: Self::Input) -> Self::Output {
            let x = self.input.process(input);
            self.output.process(x)
        }
    }

    impl<T> ConnectedSystems for T {
        fn connect<I1, OC, O2, S2: System<Input = OC, Output = O2>>(
            self,
            other: S2,
        ) -> impl System<Input = I1, Output = O2>
        where
            Self: System<Input = I1, Output = OC>,
        {
            ConnectedSystem {
                input: self,
                output: other,
            }
        }
    }
}
