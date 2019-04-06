### Synopsis

Programatically produce sound, possibly with EDSL.

### Example

See [kv545.m4a](kv545.m4a).

### Running

Make sure [portaudio](http://www.portaudio.com/) is installed, then run `cargo run --release`.

The release flag is important since
we are using quite some generators and they are slow in debug mode.
