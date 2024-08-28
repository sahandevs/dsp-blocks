use std::time::Duration;

use crate::dsp::blocks::*;

type Input = (
    (synths::OscillatorControls, synths::OscillatorControls),
    synths::OscillatorControls,
);

pub fn create_playground_blocks() -> (Input, impl Block<Input, Output = Wave>) {
    let total_dur = Duration::from_millis(1000);
    let sterio_sys = blocks::synths::Oscillator
        .join(blocks::synths::Oscillator)
        .join(blocks::synths::Oscillator)
        .connect(Basic::Mix);
    let input = (
        (
            synths::OscillatorControls {
                duration: total_dur.clone(),
                freq: 440.0,
                phase: 0f32,
                wave: synths::WaveType::Sinusoid,
            },
            synths::OscillatorControls {
                duration: total_dur.clone(),
                freq: 312.0,
                phase: 0.5f32,
                wave: synths::WaveType::Sinusoid,
            },
        ),
        synths::OscillatorControls {
            duration: total_dur.clone(),
            freq: 73.0,
            phase: 0f32,
            wave: synths::WaveType::Sinusoid,
        },
    );

    (input, sterio_sys)
}
