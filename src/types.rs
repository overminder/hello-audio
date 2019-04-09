
pub trait Sound = Iterator<Item=f32> + 'static + Send;

pub const SAMPLE_RATE: f64 = 44_100.0;
pub const HALF_STEP: f64 = 1.0595;

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

pub const JI_5: &'static [f64] = &[
    523.25,
    588.65625,
    654.0625,
    697.6666666666666,
    784.875,
    872.0833333333334,
    981.09375,
    1046.5,
];

