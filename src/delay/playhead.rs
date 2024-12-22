use super::envelope::Envelope;

fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}

#[derive(PartialEq)]
pub enum FeedbackSrc {
    Playhead,
    Grain,
}

pub struct PlayHead {
    sample_rate: f32,
    buffer_length_sec: f32,
    pub distance: f32,         // distance from record_head range 0-1
    pub current_distance: f32, // current distance interpolates to distance
    pub window_size: f32,      // window_size range between 0-1
    grain_size: f32,
    trig: Trig, // triggers grains
    pub grains: Vec<Grain>,
    grain_num: usize,
    pub feedback_src: FeedbackSrc,
    pitch: i32,
    gain: f32,
}

impl PlayHead {
    pub fn new(distance: f32, grain_num: usize) -> Self {
        PlayHead {
            sample_rate: 0.0,
            buffer_length_sec: 0.0,
            distance,
            current_distance: distance,
            window_size: 0.0,
            grain_size: 0.0,
            trig: Trig::new(),
            grain_num,
            grains: {
                let mut grains: Vec<Grain> = Vec::with_capacity(grain_num);
                for _ in 0..grain_num {
                    grains.push(Grain::default());
                }
                grains
            },
            feedback_src: FeedbackSrc::Playhead,
            pitch: 0,
            gain: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn set_feedback_src(&mut self, feedback_src: FeedbackSrc) {
        self.feedback_src = feedback_src;
    }

    pub fn set_pitch(&mut self, pitch: i32) {
        self.pitch = pitch;
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_window_size(&mut self, window_size: f32) {
        self.window_size = window_size;
    }

    pub fn set_grain_size(&mut self, grain_size: f32) {
        self.grain_size = grain_size;
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }

    pub fn init(&mut self, sample_rate: f32, buffer_length_sec: f32) {
        self.sample_rate = sample_rate;
        self.buffer_length_sec = buffer_length_sec;
        self.trig.set_sample_rate(sample_rate);
    }

    pub fn set_current_distance(&mut self) {
        if self.current_distance != self.distance {
            self.current_distance = lerp(self.current_distance, self.distance, 0.001);
        }
    }

    pub fn set_density(&mut self, density: f32) {
        self.trig.set_inc(density);
    }

    pub fn get_grain_data(&self) -> Vec<(f32, f32, f32)> {
        let mut data = Vec::new();
        for grain in self.grains.iter() {
            if grain.active {
                data.push((grain.pos, grain.gain, grain.stereo_pos));
            }
        }
        data
    }

    pub fn set_chaos(&mut self, chaos: f32) {
        self.trig.chaos = chaos;
    }

    pub fn update(&mut self) {
        self.set_current_distance();
        if self.trig.update() {
            self.activate_grain();
        }
        for grain in self.grains.iter_mut() {
            if grain.active {
                grain.update(self.pitch, self.gain);
            }
        }
    }

    pub fn get_init_gain(&self) -> f32 {
        let active_grain_num = self.grains.iter().filter(|grain| grain.active).count();
        let ratio = active_grain_num as f32 / self.grain_num as f32;

        let decay_factor = 0.7;
        let scaled_ratio = (1.0 - ratio).powf(decay_factor);

        scaled_ratio.max(0.3)
    }

    fn activate_grain(&mut self) {
        let init_gain = self.get_init_gain();
        for grain in self.grains.iter_mut() {
            if !grain.active {
                let pos = rand::random::<f32>() * 2.0 - 1.0;
                grain.activate(
                    pos,
                    (self.grain_size * self.sample_rate) as usize, // max 1sec
                    init_gain,
                    self.window_size,
                    self.distance,
                    self.sample_rate,
                    self.buffer_length_sec,
                );
                break;
            }
        }
    }
}

#[derive(Default)]
#[allow(dead_code)]
pub struct Grain {
    pub active: bool,
    pub pos: f32, // position in window -1 to 1
    pub stereo_pos: f32,
    sample_rate: f32,
    buffer_length_sec: f32,
    length: usize,
    counter: usize,
    pub gain: f32,
    init_gain: f32,
    env: Envelope,
}

impl Grain {
    fn activate(
        &mut self,
        pos: f32,
        length: usize,
        init_gain: f32,
        window_size: f32,
        playhead_distance: f32,
        sample_rate: f32,
        buffer_length_sec: f32,
    ) {
        self.pos = window_size * 0.25 * pos + playhead_distance;

        if self.pos < 0.0 {
            self.pos = 1.0 + self.pos;
        } else if self.pos > 1.0 {
            self.pos = self.pos - 1.0;
        }

        self.active = true;
        self.length = length;
        self.env.set_inc(1.0 / length as f64);
        self.stereo_pos = rand::random::<f32>() * 2.0 - 1.0;
        self.init_gain = init_gain;
        self.sample_rate = sample_rate;
        self.buffer_length_sec = buffer_length_sec;
    }

    fn update(&mut self, pitch: i32, gain: f32) {
        let pitch = (2.0f32).powf(pitch as f32 / 12.0);
        let inc = (1.0 - pitch) / (self.sample_rate * self.buffer_length_sec);

        self.pos += inc;

        if self.pos < 0.0 {
            self.pos = 1.0 + self.pos;
        } else if self.pos > 1.0 {
            self.pos = self.pos - 1.0;
        }

        self.counter += 1;
        self.gain = self.env.next_sample() as f32 * self.init_gain * gain;
        if self.counter > self.length {
            self.active = false;
            self.counter = 0;
        }
    }
}

struct Trig {
    inc: f32,
    phase: f32,
    sample_rate: f32,
    chaos: f32,
    rnd: f32,
}

impl Trig {
    fn new() -> Self {
        Trig {
            inc: 0.0,
            phase: 0.0,
            sample_rate: 0.0,
            chaos: 1.0,
            rnd: 1.0,
        }
    }

    fn set_inc(&mut self, freq: f32) {
        self.inc = freq / self.sample_rate;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn update(&mut self) -> bool {
        self.phase += self.inc;
        if self.phase > 1.0 + self.rnd {
            self.rnd = (rand::random::<f32>() - 0.5) * 2.0 * self.chaos;
            self.phase -= 1.0 + self.rnd;
            return true;
        }
        false
    }
}
