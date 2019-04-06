#![allow(dead_code)]
#![feature(generators, generator_trait)]
#![feature(trait_alias)]

mod music;
mod geniter;
mod playback;
mod to_wav;

fn main() {
    // to_wav::save(music::kv545(), "kv545.wav");
    // let m = music::kv545().collect::<Vec<_>>().into_iter();
    let m = music::kv545();
    playback::play(m).unwrap()
}
