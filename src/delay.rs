mod envelope;
mod playhead;

#[allow(dead_code)]
struct OnePole {
    alpha: f32,
    z1_l: f32,
    z1_r: f32,
}

#[allow(dead_code)]
impl OnePole {
    fn new() -> Self {
        Self {
            alpha: 0.5,
            z1_l: 0.0,
            z1_r: 0.0,
        }
    }
    fn next(&mut self, input: (f32, f32)) -> (f32, f32) {
        self.z1_l = self.z1_l + self.alpha * (input.0 - self.z1_l);
        self.z1_r = self.z1_r + self.alpha * (input.1 - self.z1_r);
        (self.z1_l, self.z1_r)
    }
}

pub struct Delay {
    data: Vec<(f32, f32)>,
    sample_rate: f32,
    write_head: usize,
    play_heads: Vec<playhead::PlayHead>,
    pub feedback: f32,
    filter: OnePole,
    feedback_sample: (f32, f32),
}

impl Delay {
    pub fn new(length: usize, sample_rate: f32) -> Self {
        let play_head_distances = [0.5, 0.25];
        let data: Vec<(f32, f32)> = vec![(0.0, 0.0); length];

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
            feedback_sample: (0.0, 0.0),
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

    fn write(&mut self, signal: (&f32, &f32)) {
        let feedback = (
            self.feedback_sample.0 * self.feedback,
            self.feedback_sample.1 * self.feedback,
        );

        let feedback = self.filter.next(feedback);

        self.data[self.write_head].0 = signal.0 + feedback.0;
        self.data[self.write_head].1 = signal.1 + feedback.1;

        self.write_head = (self.write_head + 1) % self.data.len();
    }

    fn read(&mut self, signal: (&mut f32, &mut f32)) {
        let mut out = (0.0, 0.0);
        let mut feedback = (0.0, 0.0);
        for play_head in self.play_heads.iter_mut() {
            play_head.update();

            let buffer_size = self.data.len() as f32;
            let offset = buffer_size * play_head.current_distance;

            let mut feedback_pos = self.write_head as f32 - offset;

            if feedback_pos < 0.0 {
                feedback_pos += buffer_size;
            }

            feedback.0 += self.data[feedback_pos as usize % self.data.len()].0;
            feedback.1 += self.data[feedback_pos as usize % self.data.len()].1;

            let grain_data = play_head.get_grain_data();

            grain_data.iter().for_each(|(pos, gain, stereo_pos)| {
                let abs_window_size = play_head.window_size * self.sample_rate;
                let grain_offset = abs_window_size / 2.0 * pos;

                let mut read_pos = (self.write_head as f32 - offset) + grain_offset;

                if read_pos < 0.0 {
                    read_pos += buffer_size;
                }

                let index = read_pos as usize % self.data.len();

                let left_gain = 0.5 * (1.0 - stereo_pos);
                let right_gain = 0.5 * (1.0 + stereo_pos);

                out.0 += self.data[index].clone().0 * *gain * left_gain;
                out.1 += self.data[index].clone().1 * *gain * right_gain;
            });
        }

        self.feedback_sample = feedback;
        //TBD DRY WET
        //*sample *= 0.0;
        *signal.0 += out.0;
        *signal.1 += out.1;
    }

    pub fn render(&mut self, data: (&mut f32, &mut f32)) {
        self.write((data.0, data.1));
        self.read(data);
    }
}
