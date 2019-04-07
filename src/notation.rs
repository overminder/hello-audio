use std::io::Read;
use std::str::FromStr;
use itertools::Itertools;
use lexpr::{
    Value::{self, *},
    Atom::{self, *},
};
use crate::notes::*;
use crate::types::*;

pub fn read_sheet(r: impl Read) -> Sheet {
    let v = lexpr::from_reader(r).expect("from_reader");
    read_toplevel(&v)
}

#[derive(Copy, Clone)]
enum Clef {
    Treble,
    Bass,
}

struct TrackState {
    clef: Clef,
}

impl TrackState {
    fn new() -> Self {
        Self { clef: Clef::Treble }
    }
}

fn read_toplevel(v: &Value) -> Sheet {
    let vs = expect_list(v, "toplevel");
    match vs {
        [Atom(Symbol(tag)), _beat, tracks]
            if tag == "piano" => read_piano_tracks(tracks),
        _ => panic!("Not a toplevel: {}", v),
    }
}

fn read_piano_tracks(v: &Value) -> Sheet {
    let vs = expect_list(v, "tracks");
    let mut xst = TrackState::new();
    let mut yst = TrackState::new();
    let mut xs = vec![];
    let mut ys = vec![];
    for (x, y) in vs.iter().tuples() {
        // Two tracks at a time.
        let xb = read_bar(expect_list(x, "bar"), &mut xst);
        let yb = read_bar(expect_list(y, "bar"), &mut yst);
        xs.extend(xb.into_iter());
        ys.extend(yb.into_iter());
    }
    vec![xs, ys]
}

fn read_bar(vs: &[Value], st: &mut TrackState) -> Track {
    let mut out = vec![];
    for v in vs {
        read_cmd(v, st, &mut out);
    }
    out
}

fn read_cmd(v: &Value, st: &mut TrackState, out: &mut Track) {
    match v {
        Atom(Symbol(s)) => read_simple_cmd(s, st, out),
        List(vs) => read_compound_cmd(vs, st, out),
        _ => panic!("Not a cmd: {}", v),
    }
}

fn read_simple_cmd(s: &str, st: &mut TrackState, out: &mut Track) {
    if let Some(clef) = try_read_clef(s) {
        // Is a clef change
        st.clef = clef;
    } else if let Some(dur) = try_read_duration(s) {
        // Is a rest with duration
        out.push(mk_rest(dur));
    } else {
        panic!("Unknown simple cmd: {}", s);
    }
}

fn read_compound_cmd(vs: &[Value], st: &mut TrackState, out: &mut Track) {
    let tag = vs[0].as_symbol().expect("tag");
    if tag == "^" {
        // (^ note note): slur the notes
        let mut to_slur = vec![];
        for v in &vs[1..] {
            to_slur.extend(try_read_rawnote(v, st).expect("slur")
                           .iter()
                           .map(|x| x.to_note()))
        }
        let len = to_slur.len();
        for n in &mut to_slur[..len - 1] {
            // TODO: Tweak
            n.easing = 0.1;
            n.rest_after = 0.;
        }
        out.extend(to_slur.into_iter());

    } else {
        let rns = try_read_rawnote_from_list(vs, st).expect("rawnote");
        for rn in rns {
            out.push(rn.to_note());
        }
    }
}

struct RawNote {
    dur: Duration,
    clef: Clef,
    pitch: Vec<i32>,
}

impl RawNote {
    fn to_note(&self) -> Note {
        let p = if self.pitch.is_empty() {
            Pitch::Rest
        } else {
            let fs: Vec<_> = self.pitch
                .iter()
                .map(|p| freq_for_clef(*p, self.clef))
                .collect();
            if fs.len() == 1 {
                Pitch::Single(fs[0])
            } else {
                Pitch::Chord(fs)
            }
        };
        mk_note(self.dur, p)
    }

    fn trill(&self) -> Vec<Self> {
        let n = 4;
        let dur = self.dur.faster(n);
        assert_eq!(self.pitch.len(), 1);
        let p = self.pitch[0];
        let ps = vec![p, p + 1, p, p + 1];
        ps.iter()
            .map(|p| Self { pitch: vec![*p], dur, clef: self.clef })
            .collect()
    }
}

fn try_read_rawnote(v: &Value, st: &TrackState)
    -> Option<Vec<RawNote>> {

    if let Some(vs) = as_list(v) {
        try_read_rawnote_from_list(vs, st)
    } else {
        let dur = try_read_duration(v.as_symbol()?)?;
        // Rest
        Some(vec![RawNote {
            dur,
            pitch: vec![],
            clef: st.clef,
        }])
    }
}

fn try_read_rawnote_from_list(vs: &[Value], st: &TrackState)
    -> Option<Vec<RawNote>> {

    let tag = vs[0].as_symbol()?;
    if tag == "tr" {
        // Trill
        Some(vs[1..].iter().
             flat_map(|v| {
                 let rns = try_read_rawnote(v, st).expect("tr-rawnote");
                 rns.into_iter().flat_map(|x| x.trill())
             })
             .collect())
    } else {
        // Single or chord
        let dur = try_read_duration(tag)?;
        Some(vs[1..].iter().map(|v| RawNote {
            dur,
            pitch: read_pitch(v),
            clef: st.clef,
        }).collect())
    }
}

fn read_pitch(v: &Value) -> Vec<i32> {
    if let Some(v) = v.as_i64() {
        vec![v as i32]
    } else if let Some(vs) = as_list(v) {
        // Chord
        vs.iter().map(|v| v.as_i64().expect("pitch") as i32).collect()
    } else {
        panic!("Not a pitch: {}", v)
    }
}

fn freq_for_clef(ix: i32, clef: Clef) -> f64 {
    let dix = match clef {
        Clef::Treble => 2,
        Clef::Bass => -10,
    };
    // -7 to move C5 to C4.
    let mut ix = ix + dix - 7;

    let mut pow2 = 0;
    while ix < 0 {
        ix += 7;
        pow2 -= 1;
    }
    while ix > 7 {
        ix -= 7;
        pow2 += 1;
    }
    assert!(ix >= 0 && ix <= 7);
    OCTAVE_5[ix as usize] * (2_f64.powi(pow2))
}

fn try_read_duration(s: &str) -> Option<Duration> {
    if s.starts_with("/") {
        let mut sp = &s[1..];
        let mut dots = 0;
        while sp.ends_with(".") {
            sp = &sp[..sp.len() - 1];
            dots += 1;
        }
        let klass = i32::from_str(sp).ok()?;
        Some(Duration { klass, dots })
    } else {
        None
    }
}

fn try_read_clef(v: &str) -> Option<Clef> {
    Some(match v {
        "treble-C" => Clef::Treble,
        "bass-C" => Clef::Bass,
        _ => return None,
    })
}

// Note / sexp helpers

fn mk_rest(dur: Duration) -> Note {
    mk_note(dur, Pitch::Rest)
}

fn mk_note(dur: Duration, pitch: Pitch) -> Note {
    Note {
        duration: dur,
        pitch,

        amp: 1.,
        easing: 0.05,
        rest_after: 0.1,
    }
}

fn as_list<'a>(v: &'a Value) -> Option<&'a [Value]> {
    match v {
        &List(ref vs) => Some(vs),
        _ => None,
    }
}

fn expect_list<'a>(v: &'a Value, msg: &str) -> &'a [Value] {
    as_list(v)
        .expect(&format!("Expecting {} (a list), but got {}", msg, v))
}
