use std::time::Duration;

use synths::OscillatorControls;

use crate::dsp::blocks::*;
use crate::graph::{Block, CanConnect, CanFork, CanStack};
use crate::{graph, vis};

type Input = (
    ((OscillatorControls, OscillatorControls), OscillatorControls),
    OscillatorControls,
);

pub fn create_playground_blocks() -> anyhow::Result<(Input, impl Block<Input, Output = ()>)> {
    let total_dur = Duration::from_millis(230);
    let sterio_sys = /* _ */
        blocks::synths::Oscillator.connect(vis::WaveView::small())
        .join(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .join(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .join(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .connect(Basic::Mix.connect(vis::WaveView::small()))
        .fork(
            blocks::EnvelopeBlock::default().connect(vis::WaveView::small())
            .join(blocks::EnvelopeBlock::builder().window(WindowSetting::builder().hop_length(1).build()).build().connect(vis::WaveView::small()))
        )
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
