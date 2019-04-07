use std::f64::consts::PI;
use crate::geniter::GenIter;
use itertools::Itertools;
use itertools::EitherOrBoth::{Both, Left, Right};
use crate::types::*;

pub fn sine(freq: f64, duration: f64) -> impl Sound {
    let ticks = (SAMPLE_RATE * duration) as usize;
    let step = freq / SAMPLE_RATE * 2.0 * PI;
    GenIter(move || {
        let mut x = 0_f64;
        for _ in 0..ticks {
            yield x.sin() as f32;
            x += step;
        }
    })
}

pub fn mult(x: impl Sound, y: impl Sound) -> impl Sound {
    x.zip(y).map(|(x, y)| x * y)
}

pub fn easing(e_dur: f64, duration: f64) -> impl Sound {
    let ticks = (SAMPLE_RATE * duration) as usize;
    let ease = (e_dur * SAMPLE_RATE) as usize;
    let ease_step = 1_f32 / (ease as f32);
    GenIter(move || {
        let mut out = 0.;
        for t in 0..ticks {
            if t < ease {
                out += ease_step;
            } else if t > ticks - ease {
                out -= ease_step;
            }
            yield out;
        }
    })
}

pub fn delay(duration: f64, s: impl Sound) -> impl Sound {
    let ticks = (SAMPLE_RATE * duration) as usize;
    GenIter(move || {
        for _ in 0..ticks {
            yield 0_f32;
        }
        for v in s {
            yield v;
        }
    })
}

pub fn superpos(x: impl Sound, y: impl Sound) -> impl Sound {
    x.zip_longest(y)
     .map(|xy| {
         match xy {
             Left(x) => x,
             Right(y) => y,
             Both(x, y) => x + y,
         }
     })
}
