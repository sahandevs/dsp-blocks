use std::f32::consts::PI;

pub const SR: usize = 44100;

pub type Wave = Vec<f32>;

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
