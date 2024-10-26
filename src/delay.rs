mod envelope;
mod playhead;

#[allow(dead_code)]
struct OnePole {
    alpha: f32,
    z1: f32,
}

#[allow(dead_code)]
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
    sample_rate: f32,
    write_head: usize,
    play_heads: Vec<playhead::PlayHead>,
    pub feedback: f32,
    filter: OnePole,
    feedback_sample: f32,
}

impl Delay {
    pub fn new(length: usize, sample_rate: f32) -> Self {
        let play_head_distances = [0.5, 0.25];
        let data = vec![0.0; length];

        Self {
            data,
            sample_rate,
            write_head: 0,
            feedback: 0.0,
            play_heads: play_head_distances
                .iter()
                .map(|distance| playhead::PlayHead::new(*distance, sample_rate))
                .collect(),
            filter: OnePole::new(),
            feedback_sample: 0.0,
        }
    }

    pub fn set_distance(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_distance(value);
    }

    pub fn set_density(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_density(value);
    }

    pub fn set_alpha(&mut self, value: f32) {
        self.filter.alpha = value;
    }

    fn write(&mut self, signal: &f32) {
        let feedback = self.feedback_sample * self.feedback;
        let feedback = self.filter.next(feedback);

        self.data[self.write_head] = *signal + feedback; 
        self.write_head = (self.write_head + 1) % self.data.len();
    }

    fn read(&mut self, sample: &mut f32) {
        let mut out = 0.0;
        for play_head in self.play_heads.iter_mut() {
            play_head.update();

            let buffer_size = self.data.len() as f32;

            let grain_data = play_head.get_grain_data();

            grain_data.iter().for_each(|(pos, gain)| {
                let offset = buffer_size * play_head.distance;

                let abs_window_size = play_head.window_size * self.sample_rate;
                let grain_offset = abs_window_size / 2.0 * pos;

                let mut read_pos = (self.write_head as f32 - offset) + grain_offset;

                if read_pos < 0.0 {
                    read_pos += buffer_size;
                }

                let index = read_pos as usize % self.data.len();
                out += self.data[index] * gain * 0.5;
            });
        }

        self.feedback_sample = out;
        //TBD DRY WET
        *sample *= 0.75;
        *sample += out;
    }

    pub fn render(&mut self, input: &mut f32) {
        self.write(input);
        self.read(input);
    }
}
