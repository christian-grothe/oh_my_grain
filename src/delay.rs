mod playhead;

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

pub struct Delay {
    data: Vec<f32>,
    length: usize,
    write_head: usize,
    play_heads: Vec<playhead::PlayHead>,
    pub feedback: f32,
    filter: OnePole,
}

impl Delay {
    pub fn new(length: usize) -> Self {
        let play_head_distances = [0.5];
        let data = vec![0.0; length];

        Self {
            data,
            length,
            write_head: 0,
            feedback: 0.0,
            play_heads: play_head_distances
                .iter()
                .map(|distance| playhead::PlayHead::new(*distance, 44100.0))
                .collect(),
            filter: OnePole::new(),
        }
    }

    pub fn set_distance(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_distance(value);
    }

    pub fn set_alpha(&mut self, value: f32) {
        self.filter.alpha = value;
    }

    fn write(&mut self, signal: &f32) {
        self.data[self.write_head] = *signal;
        self.write_head = (self.write_head + 1) % self.length;
    }

    fn read(&mut self, sample: &mut f32) {
        for play_head in self.play_heads.iter_mut() {
            play_head.update();
        }

        let out = 0.0;
        *sample = out;
    }

    pub fn render(&mut self, input: &mut f32) {
        self.write(input);
        self.read(input);
    }
}
