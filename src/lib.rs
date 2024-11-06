// re-export
pub use rayon;

// use
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

const RIFF: [u8; 4] = [0x52, 0x49, 0x46, 0x46];
const WAVE: [u8; 4] = [0x57, 0x41, 0x56, 0x45];

#[derive(Debug)]
pub struct Wav {
    channels: u16,
    bits_per_sample: u16,
    rate: u32,
    path_buf: PathBuf,
}

impl Wav {
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path_buf = path.as_ref().to_path_buf();

        let mut file = File::open(path)?;

        let (channels, bits_per_sample, rate) = decode_header(&mut file)?;

        Ok(Self {
            channels,
            bits_per_sample,
            rate,
            path_buf,
        })
    }

    pub fn read_samples(&mut self, dst: &mut Vec<Vec<i16>>) -> io::Result<()> {
        let mut file = File::open(&self.path_buf)?;

        let mut buffer = [0; 2];

        while file.read_exact(&mut buffer).is_ok() {
            let sample = i16::from_le_bytes(buffer);

            let i = dst[0].len() % self.channels as usize;

            dst[i].push(sample);
        }

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path_buf
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn bits_per_sample(&self) -> u16 {
        self.bits_per_sample
    }
}

fn decode_header<R: Read>(r: &mut R) -> io::Result<(u16, u16, u32)> {
    let mut buffer = [0; 44];

    r.read_exact(&mut buffer)?;

    if &buffer[0..4] != RIFF || &buffer[8..12] != WAVE {
        Err(io::Error::new(io::ErrorKind::InvalidData, "not a wav file"))?;
    }

    // channels, bits per sample, rate
    Ok((
        u16::from_le_bytes([buffer[22], buffer[23]]),
        u16::from_le_bytes([buffer[34], buffer[35]]),
        u32::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
    ))
}

fn detect_peaks_with_slope(volumes: &[f64], threshold: f64) -> Vec<usize> {
    fn is_peak(volumes: &[f64], index: usize, threshold: f64) -> bool {
        volumes[index] > threshold
            && volumes[index] > volumes[index - 1]
            && volumes[index] > volumes[index + 1]
    }

    (1..volumes.len() - 1)
        .into_par_iter()
        .filter(|&i| is_peak(volumes, i, threshold))
        .collect()
}

fn calculate_tempus(peaks: &[usize], rate: u32) -> f64 {
    calculate_average_interval(peaks)
        .map(|avg_interval| (rate as f64 / avg_interval) * 60.0)
        .unwrap_or(0.0)
}

fn calculate_average_interval(peaks: &[usize]) -> Option<f64> {
    if peaks.len() < 2 {
        None?
    }

    let total_interval: usize = peaks.windows(2).map(|w| w[1] - w[0]).sum();

    Some(total_interval as f64 / (peaks.len() - 1) as f64)
}

fn calculate_sample_volume(sample: i16) -> f64 {
    match sample {
        i16::MIN => i16::MAX as f64,

        0 => 0.0,

        _ => (sample.abs() as f64).log10() * 20.0,
    }
}

pub fn smooth_volumes(samples: &[i16], window_size: usize) -> Vec<f64> {
    let mut volumes = Vec::with_capacity(samples.len());

    let mut sum = 0.0;

    for i in 0..samples.len() {
        sum += calculate_sample_volume(samples[i]);

        if i >= window_size {
            sum -= calculate_sample_volume(samples[i - window_size]);
        }

        let current_window_size = if i + 1 < window_size {
            i + 1
        } else {
            window_size
        };

        volumes.push(sum / current_window_size as f64);
    }

    volumes
}

pub fn process_chunk(samples: &[i16], rate: u32, threshold: f64) -> f64 {
    calculate_tempus(
        &detect_peaks_with_slope(&smooth_volumes(samples, 100), threshold),
        rate,
    )
}
