struct OnePole {
    alpha: f32,
    z1: f32,
}

impl OnePole {
    fn new() -> Self {
        Self {
            alpha: 0.5,
            z1: 0.0,
        }
    }

    fn next(&mut self, input: f32) -> f32 {
        self.z1 = self.z1 + self.alpha * (input - self.z1);
        self.z1
    }
}

struct PlayHead {
    distance: f32,
    gain: f32,
    feedback: f32,
}

pub struct Delay {
    data: Vec<f32>,
    length: usize,
    write_head: usize,
    play_heads: Vec<PlayHead>,
    pub feedback: f32,
    filter: OnePole,
}

impl Delay {
    pub fn new(length: usize) -> Self {
        let play_head_params = [
            //  dist  gain  feedback
            (0.1, 0.8, 0.3),
            (0.1, 0.6, 0.2),
            (0.1, 0.7, 0.4),
            (0.1, 0.3, 0.3),
        ];

        Self {
            data: vec![0.0; length],
            length,
            write_head: 0,
            feedback: 0.0,
            play_heads: play_head_params
                .iter()
                .map(|(distance, gain, feedback)| PlayHead {
                    distance: *distance,
                    gain: *gain,
                    feedback: *feedback,
                })
                .collect(),
            filter: OnePole::new(),
        }
    }

    pub fn set_distance(&mut self, index: usize, value: f32) {
        self.play_heads[index].distance = value;
    }

    pub fn set_alpha(&mut self, value: f32) {
        self.filter.alpha = value;
    }

    fn write(&mut self, signal: &f32) {
        let mut feedback_sample = 0.0;

        for play_head in self.play_heads.iter() {
            let offset = (self.data.len() as f32 * play_head.distance) as usize;
            let pos = (self.write_head + self.data.len() - offset) % self.data.len();

            feedback_sample += self.data[pos] * self.feedback * play_head.feedback;
        }

        feedback_sample = self.filter.next(feedback_sample);

        self.data[self.write_head] = signal + feedback_sample;
        self.write_head = (self.write_head + 1) % self.length;
    }

    fn read(&mut self,  sample: &mut f32) {
        let mut out = 0.0;
        for play_head in self.play_heads.iter() {
            let offset = (self.data.len() as f32 * play_head.distance) as usize;
            let pos = (self.write_head + self.data.len() - offset) % self.data.len();

            let sample = self.data[pos];
            out += sample * play_head.gain;
        }
        *sample = out;
    }

    pub fn render(&mut self, input: &mut f32) {
        self.write(input);
        self.read(input);
    }
}
