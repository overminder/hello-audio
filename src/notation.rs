use std::io::Read;
use std::str::FromStr;
use std::collections::HashMap;
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

#[derive(Clone)]
struct TrackState {
    clef: Clef,
    // Map from pitch to number of sharps.
    sharps: HashMap<i32, i32>,
    global_sharp: i32,
}

impl TrackState {
    fn new() -> Self {
        Self {
            clef: Clef::Treble,
            sharps: HashMap::new(),
            global_sharp: 0,
        }
    }

    fn add_sharp(&mut self, ix: i32, n: i32) {
        if let Some(orig) = self.sharps.get_mut(&ix) {
            *orig += n;
        } else {
            self.sharps.insert(ix, n);
        }
    }

    fn reset_sharp(&mut self, ix: i32) {
        self.sharps.remove(&ix);
    }

    fn norm_pitch(&self, ix: i32) -> i32 {
        let dix = match self.clef {
            Clef::Treble => 2,
            Clef::Bass => -10,
        };
        // -7 to move C5 to C4.
        ix + dix - 7
    }

    fn freq_for_ix(&self, mut ix: i32) -> f64 {
        let sharp = self.sharps.get(&ix).cloned().unwrap_or(0)
            + self.global_sharp;

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
        OCTAVE_5[ix as usize]
            * (2_f64.powi(pow2))
            * (HALF_STEP.powi(sharp))
    }
}

fn read_toplevel(v: &Value) -> Sheet {
    let vs = expect_list(v, "toplevel");
    match vs {
        [Atom(Symbol(tag)), _beat, gsharp, tracks]
            if tag == "piano" =>
                read_piano_tracks(tracks, gsharp.as_i64().unwrap()),
        _ => panic!("Not a toplevel: {}", v),
    }
}

fn read_piano_tracks(v: &Value, gsharp: i64) -> Sheet {
    let vs = expect_list(v, "tracks");
    let mut xst = TrackState::new();
    xst.global_sharp = gsharp as i32;
    let mut yst = TrackState::new();
    yst.global_sharp = gsharp as i32;
    let mut xs = vec![];
    let mut ys = vec![];
    let mut to_drop = 0;
    for (x, y) in vs.iter().tuples() {
        if x.as_symbol() == Some("drop") {
            // Drop several bars.
            to_drop += y.as_i64().unwrap();
            continue;
        }

        if to_drop > 0 {
            to_drop -= 1;
            continue;
        }

        // Reset pitch for each bar.
        xst.sharps.clear();
        yst.sharps.clear();

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
    if let Some(tags) = as_list(&vs[0]) {
        let tag = tags[0].as_symbol().expect("tag");
        let rest = tags[1].as_symbol().expect("tag rest");
        if tag == "acciac" && rest == "sharp" {
            let rns = try_read_rawnote(
                &vs[1], st).expect("rawnote");
            assert_eq!(rns.len(), 1);
            let rn = &rns[0];
            let mut rns = rn.acciaccatura();
            let p = rns[0].pitch[0];
            st.add_sharp(p, 1);
            rns[0].state = st.clone();
            out.extend(rns.into_iter().map(|x| x.to_note()));
        } else {
            panic!("Unknown tag: {}", vs[0]);
        }
        return;
    }

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

    } else if tag == "staccato" {
        let mut to_slur = vec![];
        for v in &vs[1..] {
            to_slur.extend(try_read_rawnote(v, st).expect("staccato")
                           .iter()
                           .map(|x| x.to_note()))
        }
        let len = to_slur.len();
        for n in &mut to_slur[..len - 1] {
            // TODO: Tweak
            n.rest_after = 0.5;
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
    state: TrackState,
    pitch: Vec<i32>,
}

impl RawNote {
    fn to_note(&self) -> Note {
        let p = if self.pitch.is_empty() {
            Pitch::Rest
        } else {
            let fs: Vec<_> = self.pitch
                .iter()
                .map(|p| self.state.freq_for_ix(*p))
                .collect();
            if fs.len() == 1 {
                Pitch::Single(fs[0])
            } else {
                Pitch::Chord(fs)
            }
        };
        mk_note(self.dur, p)
    }

    fn trill(&self, chunks: i32) -> Vec<Self> {
        let n = chunks;
        let dur = self.dur.faster(n as usize);
        assert_eq!(self.pitch.len(), 1);
        let p = self.pitch[0];
        let ps0 = vec![p, p + 1];
        let ps = (0..(n / 2)).flat_map(|_| ps0.clone());
        ps.map(|p| Self {
                pitch: vec![p],
                dur,
                state: self.state.clone(),
            })
            .collect()
    }

    // XXX: Kind of hard to render this right.
    fn acciaccatura(&self) -> Vec<Self> {
        let n = 2;
        let mut dur = self.dur.faster(12);
        let mut dur2 = dur.slower(11);
        assert_eq!(self.pitch.len(), 1);
        let p = self.pitch[0];
        let mut st = self.state.clone();
        vec![
            Self {
                pitch: vec![p - 1],
                dur,
                state: st.clone()
            },
            Self {
                pitch: vec![p],
                dur: dur2,
                state: st.clone()
            },
        ]
    }
}

fn try_read_rawnote(v: &Value, st: &mut TrackState)
    -> Option<Vec<RawNote>> {

    if let Some(vs) = as_list(v) {
        try_read_rawnote_from_list(vs, st)
    } else {
        let dur = try_read_duration(v.as_symbol()?)?;
        // Duration only: is a rest
        Some(vec![RawNote {
            dur,
            pitch: vec![],
            state: st.clone(),
        }])
    }
}

fn try_read_rawnote_from_list(vs: &[Value], st: &mut TrackState)
    -> Option<Vec<RawNote>> {

    let tag = vs[0].as_symbol()?;
    if tag.starts_with("tr") {
        let chunks = i32::from_str(&tag[2..]).unwrap_or(4);
        // Trill
        Some(vs[1..].iter().
             flat_map(|v| {
                 let rns = try_read_rawnote(v, st).expect("tr-rawnote");
                 rns.into_iter().flat_map(|x| x.trill(chunks))
             })
             .collect())

    } else if tag == "acciac" {
        // Acciaccatura
        Some(vs[1..].iter().
             flat_map(|v| {
                 let rns = try_read_rawnote(v, st).expect("tr-rawnote");
                 rns.into_iter().flat_map(|x| x.acciaccatura())
             })
             .collect())

    } else {
        // Single or chord
        let dur = try_read_duration(tag)?;
        Some(vs[1..].iter().map(|v| RawNote {
            dur,
            pitch: read_pitch(v, st),
            state: st.clone(),
        }).collect())
    }
}

fn read_norm_simple_pitch(v: &Value, st: &TrackState) -> Option<i32> {
    Some(st.norm_pitch(v.as_i64()? as i32))
}

// These are the notes that happen in the same time.
// Empty means rest.
fn read_pitch(v: &Value, st: &mut TrackState) -> Vec<i32> {
    if let Some("r") = v.as_symbol() {
        vec![]
    } else if let Some(p) = read_norm_simple_pitch(v, st) {
        vec![p]
    } else if let Some(vs) = as_list(v) {
        // Either chord (1 2 3), or accidental (sharp 1)
        if let Some(tag) = vs[0].as_symbol() {
            let p = read_norm_simple_pitch(&vs[1], st).unwrap();
            if tag == "sharp" {
                st.add_sharp(p, 1);
            } else if tag == "flat" {
                st.add_sharp(p, -1);
            } else if tag == "natural" {
                st.reset_sharp(p);
            } else {
                panic!("read_pitch, tag = {}", tag);
            }
            vec![p]
        } else {
            // Chord
            vs.iter()
                .map(|v| read_norm_simple_pitch(v, st))
                .map(|v| v.expect("pitch"))
                .collect()
        }
    } else {
        panic!("Not a pitch: {}", v)
    }
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
        Some(Duration { klass, dots, longer: None })
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
