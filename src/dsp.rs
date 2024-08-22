use std::f32::consts::PI;
pub const SR: usize = 44100;

pub type Wave = Vec<f32>;

pub trait Block<Input> {
    type Output;

    fn process(&mut self, input: Input) -> Self::Output;
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
        pub use super::*;

        pub struct Oscillator;

        pub enum WaveType {
            Sinusoid,
        }
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
        }
    }

    pub enum Basic<const N: usize> {
        Mix,
    }

    impl<const N: usize> Block<[Wave; N]> for Basic<N> {
        type Output = Wave;

        fn process(&mut self, input: [Wave; N]) -> Self::Output {
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

    pub struct Channels<const N: usize>;

    impl<const N: usize, T: IntoArray<[Wave; N]>> Block<T> for Channels<N> {
        type Output = [Wave; N];

        fn process(&mut self, x: T) -> Self::Output {
            x.into()
        }
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
