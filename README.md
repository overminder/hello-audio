### Synopsis

Programatically produce sound, possibly with EDSL.

### Example

[kv545.m4a](kv545.m4a) is produced from [kv545.ss](kv545.ss),
an sexpr based notation for piano sheets. (I know that there are lots
of digital music formats such as MIDI and MusicXML, I just want
to learn and implmenet a bit of music theory by myself).

### Running

Playback requires [portaudio](http://www.portaudio.com/) to be installed.
OTOH, generation of WAV files is done in pure Rust.

The default behavior, `cargo run --release`, is to play the sound
syntheized from kv545.ss.

The release flag is important since
we are using quite some iterators and they are slow in debug mode.
