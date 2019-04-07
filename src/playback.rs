use portaudio as pa;
use crate::types::Sound;

const CHANNELS: i32 = 1;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;

pub fn play(mut sound: impl Sound) -> Result<(), pa::Error> {
    let pa = pa::PortAudio::new()?;

    let mut settings = pa.default_output_stream_settings(
        CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    settings.flags = pa::stream_flags::CLIP_OFF;

    let callback = move |args: pa::OutputStreamCallbackArgs<_>| {
        let buffer = args.buffer;

        for b in buffer {
            if let Some(v) = sound.next() {
                *b = v;
            } else {
                return pa::Complete
            }
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    while stream.is_active()? {
        pa.sleep(100);
    }

    stream.stop()?;
    stream.close()?;

    println!("Done playback");

    Ok(())
}
