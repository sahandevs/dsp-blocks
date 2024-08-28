use std::time::Duration;

use synths::OscillatorControls;

use crate::dsp::blocks::*;
use crate::vis;

type Input = (
    ((OscillatorControls, OscillatorControls), OscillatorControls),
    OscillatorControls,
);

pub fn create_playground_blocks() -> anyhow::Result<(Input, impl Block<Input, Output = ()>)> {
    let total_dur = Duration::from_millis(1000);
    let sterio_sys = blocks::synths::Oscillator
        .join(blocks::synths::Oscillator.connect(vis::WaveView::Small))
        .join(blocks::synths::Oscillator)
        .connect(Basic::Mix)
        .join(blocks::synths::Oscillator)
        .connect(Basic::Mix)
        .connect(vis::AudioSink::try_default()?)
        .connect(vis::WaveView::Grow)
        .connect(blocks::Discard);
    let input = (
        (
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
        ),
        synths::OscillatorControls {
            duration: total_dur.clone(),
            freq: 123.0,
            phase: 0f32,
            wave: synths::WaveType::Sinusoid,
        },
    );

    Ok((input, sterio_sys))
}
