use crate::types::*;
use crate::soundprim::*;

pub type Sheet = Vec<Track>;
pub type Track = Vec<Note>;

pub fn build_sheet(sh: &Sheet) -> impl Sound {
    let mut ss: Vec<Box<dyn Sound>> = sh
        .iter()
        // Not sure why we can't box here in the map function.
        .map(|t| build_track(t))
        .collect();
    let last = ss.pop().unwrap();
    ss.into_iter().fold(last, |x, y| Box::new(superpos(x, y)))
}

pub fn build_track(tr: &Track) -> Box<dyn Sound> {
    let mut b = Builder {
        res: None,
        t: 0.,
        bpm: 120,
    };

    b.build(tr);
    Box::new(b.res.unwrap())
}

#[derive(Copy, Clone)]
pub struct Duration {
    // duration = 1/klass. 2 = half, 4 = quad, 8 = eighth
    pub klass: i32,
    pub dots: i8,
}

impl Duration {
    fn dur(&self) -> f64 {
        2.0 / (self.klass as f64) * 1.5_f64.powi(self.dots as i32)
    }

    pub fn faster(&self, x: usize) -> Self {
        Self {
            klass: self.klass * x as i32,
            dots: self.dots,
        }
    }
}

pub enum Pitch {
    Rest,
    Single(f64),
    Chord(Vec<f64>),
}

impl Pitch {
    fn is_rest(&self) -> bool {
        match self {
            &Pitch::Rest => true,
            _ => false,
        }
    }

    fn as_chord(&self) -> Option<&[f64]> {
        match self {
            &Pitch::Chord(ref xs) => Some(xs),
            _ => None,
        }
    }

    fn as_single(&self) -> Option<f64> {
        match self {
            &Pitch::Single(ref x) => Some(*x),
            _ => None,
        }
    }
}

pub struct Note {
    // Full duration, including easing and rest-after
    pub duration: Duration,
    pub pitch: Pitch,
    pub amp: f32,

    // These are defined as percentage of duration
    pub easing: f64,
    pub rest_after: f64,
}

impl Note {
    fn is_rest(&self) -> bool {
        self.pitch.is_rest()
    }

    fn as_chord(&self) -> Option<&[f64]> {
        self.pitch.as_chord()
    }

    fn as_single(&self) -> Option<f64> {
        self.pitch.as_single()
    }

    fn dur(&self) -> f64 {
        self.duration.dur()
    }
}

struct Builder {
    res: Option<Box<Sound>>,
    t: f64,
    bpm: usize,
}

impl Builder {
    fn build(&mut self, ns: &[Note]) {
        for n in ns.iter() {
            if n.is_rest() {
                // Do nothing
            } else if let Some(ps) = n.as_chord() {
                for p in ps {
                    self.build_p(n, *p);
                }
            } else {
                self.build_p(n, n.as_single().unwrap());
            }

            self.t += n.dur() * self.dur_factor();
        }
    }

    fn dur_factor(&self) -> f64 {
        120. / self.bpm as f64
    }

    fn build_p(&mut self, n: &Note, freq: f64) {
        let dur = n.dur() * self.dur_factor();
        let sleep = dur * n.rest_after;
        let ease = dur * n.easing;
        let note_dur = dur - sleep;
        let amp = n.amp;

        let thiz = mult(sine(freq, note_dur), easing(ease, note_dur))
            .map(move |x| x * amp); 
        if let Some(v) = self.res.take() {
            self.res = Some(Box::new(superpos(v, delay(self.t, thiz))));
        } else {
            self.res = Some(Box::new(thiz));
        }
    }
}
