use std::{fs::File, io::Write, path::Path};

use crate::{
    dsp::{self, Wave},
    graph::Block,
    vis::{visualize_simple_box, DrawContext, VisualizeResult},
};

#[derive(Debug)]
pub struct WavWriter {
    file: File,
    path: String,
}

impl WavWriter {
    pub fn new<T: AsRef<Path>>(path: T) -> anyhow::Result<Self> {
        let _ = std::fs::remove_file(&path);

        Ok(Self {
            path: path.as_ref().to_str().unwrap_or_default().to_string(),
            file: std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(path)?,
        })
    }
}

impl Block<Wave> for WavWriter {
    type Output = Wave;

    #[rustfmt::skip]
    fn process(&mut self, input: Wave) -> Self::Output {
        // RIFF header
        self.file.write(b"RIFF").unwrap();
        self.file.write(&((36 + input.len() * 4) as u32).to_le_bytes()).unwrap();
        self.file.write(b"WAVE").unwrap();

        // FMT header
        self.file.write(b"fmt ").unwrap();
        self.file.write(&16u32.to_le_bytes()).unwrap();
        self.file.write(&0x3u16.to_le_bytes()).unwrap(); /* IEEE float PCM  */
        self.file.write(&1u16.to_le_bytes()).unwrap(); /* mono  */
        self.file.write(&(dsp::SR as u32).to_le_bytes()).unwrap();
        self.file.write(&(((1 * dsp::SR * 4) / 8) as i32).to_le_bytes()).unwrap();
        self.file.write(&(((1 * 4) / 8) as i16).to_le_bytes()).unwrap();
        self.file.write(&(32i16).to_le_bytes()).unwrap();

        // data header
        self.file.write(b"data").unwrap();
        self.file.write(&((input.len() * 4) as u32).to_le_bytes()).unwrap();

        let data: Vec<_> = input.iter().map(|x| x.to_le_bytes()).flatten().collect();
        self.file.write(data.as_slice()).unwrap();
        if data.len() % 2 == 1 {
          self.file.write(b"\0").unwrap();
        }

        input
    }

    fn process_and_visualize(
        &mut self,
        input: Wave,
        context: &mut DrawContext,
    ) -> (Self::Output, VisualizeResult) {
        let out = self.process(input);
        visualize_simple_box(context, &self.path, out)
    }
}
