use std::thread;
use std::sync::mpsc::channel;
use itertools::Itertools;

use crate::{
    playback,
    types::*,
};

pub fn buffer_playback(s: impl Sound) {
	let (tx, rx) = channel();
    thread::spawn(move|| {
        for vs in s.chunks(1024).into_iter() {
            let v: Vec<f32> = vs.collect();
            tx.send(v).unwrap();
        }
    });

    let bufs = rx.into_iter().flat_map(|v| v.into_iter());
    playback::play(bufs).unwrap();
}

