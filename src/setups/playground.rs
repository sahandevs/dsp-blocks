use std::time::Duration;

use synths::OscillatorControls;

use crate::dsp::blocks::*;
use crate::vis;

type Input = (
    ((OscillatorControls, OscillatorControls), OscillatorControls),
    OscillatorControls,
);

pub fn create_playground_blocks() -> anyhow::Result<impl Block<(), Output = ()>> {
    let total_dur = Duration::from_millis(100);
    let sterio_sys = blocks::synths::Oscillator
        .connect(vis::WaveView::grow())
        .join(blocks::synths::Oscillator.connect(vis::WaveView::grow()))
        .join(blocks::synths::Oscillator.connect(vis::WaveView::grow()))
        .connect(Basic::Mix.connect(vis::WaveView::grow()))
        .join(blocks::synths::Oscillator.connect(vis::WaveView::grow()))
        .connect(Basic::Mix)
        .connect(vis::WaveView::grow())
        // .connect(vis::AudioSink::try_default()?)
        .connect(blocks::Discard);
    let input = (
        (
            (
                synths::OscillatorControls {
                    duration: total_dur.clone(),
                    freq: 27.5f32, // A0
                    phase: 0f32,
                    wave: synths::WaveType::Sinusoid,
                },
                synths::OscillatorControls {
                    duration: total_dur.clone(),
                    freq: 20.6f32, // E0
                    phase: 0f32,
                    wave: synths::WaveType::Square,
                },
            ),
            synths::OscillatorControls {
                duration: total_dur.clone(),
                freq: 17.32f32, // C#0
                phase: 0f32,
                wave: synths::WaveType::Triangle,
            },
        ),
        synths::OscillatorControls {
            duration: total_dur.clone(),
            freq: 27.5f32 * 2f32,
            phase: 0f32,
            wave: synths::WaveType::Sawtooth,
        },
    );

    let out = blocks::DefaultInput(Box::new(move || input.clone())).connect(sterio_sys);
    Ok(out)
}
