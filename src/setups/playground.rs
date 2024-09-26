use std::time::Duration;

use synths::OscillatorControls;

use crate::dsp::blocks::*;
use crate::graph::{Block, CanConnect, CanFork, CanStack, MetadataExt};
use crate::vis::{Identity, WaveView};
use crate::{graph, vis};

type Input1 = (
    ((OscillatorControls, OscillatorControls), OscillatorControls),
    OscillatorControls,
);

type Input2 = (((Duration, Duration), Duration), Duration);

pub fn create_playground_blocks(
) -> anyhow::Result<((Input1, Input2), impl Block<(Input1, Input2), Output = ()>)> {
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
    let sys_1 = /* _ */
        blocks::synths::Oscillator.connect(vis::WaveView::small())
        .stack(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .stack(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .stack(blocks::synths::Oscillator.connect(vis::WaveView::small()))
        .connect(Basic::Mix)
        .fork(envelope)
        .connect(vis::WaveView::grow())
        .colored();
    let input_sys_1 = (
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

    let sys_2 = blocks::synths::KroneckerDelta::End
        .connect(ConstMultiplier(-1f32))
        .stack(blocks::synths::KroneckerDelta::Center)
        .stack(blocks::synths::KroneckerDelta::Start)
        .connect(blocks::Basic::Mix)
        .connect(WaveView::small())
        .fork(
            blocks::EnvelopeBlock::builder()
                .window(WindowSetting::builder().hop_length(1).build())
                .build()
                .connect(vis::WaveView::small())
                .stack(Identity),
        )
        .stack(blocks::synths::HeavisideStep.connect(WaveView::small()))
        .connect(AutoPad::Start)
        .connect(blocks::Basic::Mix)
        .connect(WaveView::small());

    let input_sys_2 = (
        ((total_dur.clone(), total_dur.clone()), total_dur.clone()),
        total_dur / 8,
    );

    let out_sys = sys_1
        .stack(sys_2)
        // .connect(vis::AudioSink::try_default()?)
        .connect(graph::Discard);

    Ok(((input_sys_1, input_sys_2), out_sys))
}
