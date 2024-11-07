use wav_rs::rayon::prelude::*;

fn main() {
    let mut wav = wav_rs::Wav::open("./test.wav").expect("failed to open file");

    let mut dst = vec![Vec::new(); wav.channels() as usize];

    wav.read_samples(&mut dst).expect("failed to read samples");

    let samples_per_chunk = wav.rate() as usize;

    let samples: Vec<f64> = dst[0]
        .chunks(samples_per_chunk)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|chunk| wav_rs::process_chunk(chunk, wav.rate(), 10.0))
        .collect();

    for tempo in samples {
        println!("BPM: {:.2}", tempo);
    }

    let window_size = 100;
    let smoothed_volumes = wav_rs::smooth_volumes(&dst[0], window_size);

    for volume in smoothed_volumes {
        println!("Volume(dB): {:.2}", volume);
    }
}
