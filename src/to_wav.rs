use crate::music::Sound;

pub fn save(s: impl Sound, name: &str) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let amplitude = i16::max_value() as f32;

    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for v in s {
        writer.write_sample((v * amplitude) as i16).unwrap();
    }
}
