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

    let envelope = vis::Identity
        .connect(vis::Identity)
        .connect(vis::WaveView::small())
        .stack(blocks::EnvelopeBlock::default().connect(vis::WaveView::small()))
        .stack(
            blocks::EnvelopeBlock::builder()
                .window(WindowSetting::builder().hop_length(1).build())
                .build()
                .connect(vis::WaveView::small()),
        )
        .stack(
            blocks::EnvelopeBlock::builder()
                .window(WindowSetting::builder().hop_length(128).build())
                .build()
                .connect(vis::WaveView::small()),
        );
    let main = /* _ */
        blocks::synths::Oscillator.connect(vis::WaveView::small())
        .stack(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .stack(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .stack(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .connect(Basic::Mix)
        .fork(envelope)
        .connect(vis::WaveView::grow());
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

    Ok((
        input,
        main
            // .connect(vis::AudioSink::try_default()?)
            .connect(graph::Discard),
    ))
}
