use rodio::Source;
use hound;

pub fn write_audio(path: &String, source: &mut dyn Source<Item = f32>) {

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(path, spec).unwrap();

    for s in source {
        let _ = writer.write_sample(s);
    }

}
