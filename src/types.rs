
pub trait Sound = Iterator<Item=f32> + 'static;

pub const SAMPLE_RATE: f64 = 44_100.0;

// C5...C6
pub const OCTAVE_4: &'static [f64] = &[
    523.25,
    587.33,
    659.25,
    698.46,
    783.99,
    880.00,
    987.77,
    1046.50,
];

pub const OCTAVE_5: &'static [f64] = &[
    523.25,
    587.33,
    659.25,
    698.46,
    783.99,
    880.00,
    987.77,
    1046.50,
];
