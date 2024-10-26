use std::f64::consts::PI;

#[derive(Default)]
pub struct Envelope {
    inc: f64,
    phase: f64,
    sin0: f64,
    sin1: f64,
    dsin: f64,
}

impl Envelope {
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.sin0 = (self.phase * 2.0 * PI).sin();
        self.sin1 = ((self.phase - self.inc) * 2.0 * PI).sin();
        self.dsin = 2.0 * (self.inc * 2.0 * PI).cos();
    }

    pub fn set_inc(&mut self, inc: f64) {
        self.inc = inc;
        self.reset();
    }

    pub fn next_sample(&mut self) -> f64 {
        let sinx = self.dsin * self.sin0 - self.sin1;
        self.sin1 = self.sin0;
        self.sin0 = sinx;
        sinx
    }
}
