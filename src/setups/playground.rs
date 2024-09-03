use std::time::Duration;

use synths::OscillatorControls;

use crate::dsp::blocks::*;
use crate::graph::{Block, CanConnect, CanStack};
use crate::{graph, vis};

type Input = (
    ((OscillatorControls, OscillatorControls), OscillatorControls),
    OscillatorControls,
);

pub fn create_playground_blocks() -> anyhow::Result<(Input, impl Block<Input, Output = ()>)> {
    let total_dur = Duration::from_millis(100);
    let sterio_sys = blocks::synths::Oscillator
        .connect(vis::WaveView::small())
        .join(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .connect(Basic::Diff.connect(vis::WaveView::small()))
        .join(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .connect(Basic::Mix.connect(vis::WaveView::small()))
        .join(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .connect(Basic::Amp)
        .connect(vis::WaveView::grow())
        // .connect(vis::AudioSink::try_default()?)
        .connect(graph::Discard);
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

    Ok((input, sterio_sys))
}
