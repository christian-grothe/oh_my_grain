#[allow(dead_code)]
pub struct PlayHead {
    sample_rate: f32,
    distance: f32,    // distance from record_head range 0-1
    window_size: f32, // window_size relative to sample_rate
    grain_size: f32,  // grain_size relative to window_size
    trig: Trig,       // triggers grains
    grains: [Grain; 10],
}

impl PlayHead {
    pub fn new(distance: f32, sample_rate: f32) -> Self {
        PlayHead {
            sample_rate,
            distance,
            window_size: 0.25,
            grain_size: 0.25,
            trig: Trig::new(sample_rate),
            grains: {
                let mut grains: [Grain; 10] = Default::default();
                for grain in &mut grains {
                    *grain = Grain::default()
                }
                grains
            },
        }
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }

    pub fn update(&mut self) {
        if self.trig.update() {
            self.activate_grain();
        }
        for grain in self.grains.iter_mut() {
            if grain.active {
                grain.update();
            }
        }
    }

    fn activate_grain(&mut self) {
        for grain in self.grains.iter_mut() {
            if !grain.active {
                let pos = rand::random::<f32>() * 2.0 - 1.0;
                grain.activate(
                    pos,
                    (self.window_size * self.grain_size * self.sample_rate) as usize,
                );
                break;
            }
        }
    }
}

#[derive(Default)]
#[allow(dead_code)]
struct Grain {
    active: bool,
    pos: f32, // position in window -1 to 1
    length: usize,
    counter: usize,
}

impl Grain {
    fn activate(&mut self, pos: f32, length: usize) {
        self.active = true;
        self.length = length;
        self.pos = pos;
    }

    fn update(&mut self) {
        self.counter += 1;
        nih_plug::nih_log!("{:?}", self.counter);
        if self.counter > self.length {
            self.active = false;
            self.counter = 0;
        }
    }
}

struct Trig {
    inc: f32,
    phase: f32,
}

impl Trig {
    fn new(sample_rate: f32) -> Self {
        Trig {
            inc: 2.0 / sample_rate,
            phase: 0.0,
        }
    }

    fn update(&mut self) -> bool {
        self.phase += self.inc;
        if self.phase > 1.0 {
            nih_plug::nih_log!("{:?}", self.phase);
            self.phase -= 1.0;
            return true;
        }
        false
    }
}
