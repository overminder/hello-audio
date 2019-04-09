#![allow(dead_code)]
#![feature(generators, generator_trait)]
#![feature(trait_alias)]

mod geniter;
mod playback;
mod conc;
mod to_wav;
mod notes;
mod notes_old;
mod types;
mod soundprim;
mod notation;

fn play_sheet() {
    use std::fs::File;

    let sh = notation::read_sheet(File::open("kv545.ss").unwrap());
    let m = notes::build_sheet(&sh).map(|x| x * 0.1);
    // to_wav::save(m, "kv545.wav");
    // let m = m.collect::<Vec<_>>().into_iter();
    // playback::play(m).unwrap();
    conc::buffer_playback(m);
}

fn main() {
    // to_wav::save(music::kv545(), "kv545.wav");
    // let m = music::kv545().collect::<Vec<_>>().into_iter();
    // playback::play(notes_old::kv545()).unwrap();
    play_sheet();
}
