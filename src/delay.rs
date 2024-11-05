use std::sync::{Arc, RwLock};

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

pub struct Buffer {
    pub data: Vec<(f32, f32)>,
    write_head: usize,
}

pub struct Delay {
    pub buffer: Arc<RwLock<Buffer>>,
    pub sample_rate: f32,
    play_heads: Vec<playhead::PlayHead>,
    pub feedback: f32,
    filter: OnePole,
    feedback_sample: (f32, f32),
}

impl Delay {
    pub fn new(play_heads: usize, grain_num: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Buffer{
                data: vec![(0.0, 0.0); 1024],
                write_head:0,
            })),
            sample_rate: 0.0,
            feedback: 0.0,
            play_heads: (0..play_heads)
                .map(|_| playhead::PlayHead::new(0.5, grain_num))
                .collect(),
            filter: OnePole::new(),
            feedback_sample: (0.0, 0.0),
        }
    }

    pub fn init(&mut self, buffer_length: usize, sample_rate: f32) {
        nih_plug::nih_log!("init delay, {:?}", buffer_length);
        let mut buffer = self.buffer.write().unwrap();
        buffer.data.resize(buffer_length, (0.0, 0.0));

        self.sample_rate = sample_rate;
        self.play_heads.iter_mut().for_each(|play_head| {
            play_head.set_sample_rate(sample_rate);
        });
    }

    pub fn set_distance(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_distance(value);
    }

    pub fn set_density(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_density(value);
    }

    pub fn set_window_size(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_window_size(value);
    }

    pub fn set_grain_size(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_grain_size(value);
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
        let mut buffer = self.buffer.write().unwrap();
        let write_head = buffer.write_head;

        buffer.data[write_head].0 = signal.0 + feedback.0;
        buffer.data[write_head].1 = signal.1 + feedback.1;

        buffer.write_head = (buffer.write_head + 1) % buffer.data.len();
    }

    fn read(&mut self, signal: (&mut f32, &mut f32)) {
        let mut out = (0.0, 0.0);
        let mut feedback = (0.0, 0.0);
        for play_head in self.play_heads.iter_mut() {
            play_head.update();

            let buffer = self.buffer.read().unwrap();

            let buffer_size = buffer.data.len() as f32;
            let offset = buffer_size * play_head.current_distance;

            let mut feedback_pos = buffer.write_head as f32 - offset;

            if feedback_pos < 0.0 {
                feedback_pos += buffer_size;
            }

            feedback.0 += buffer.data[feedback_pos as usize % buffer.data.len()].0;
            feedback.1 += buffer.data[feedback_pos as usize % buffer.data.len()].1;

            let grain_buffer = play_head.get_grain_data();

            grain_buffer.iter().for_each(|(pos, gain, stereo_pos)| {
                let abs_window_size = play_head.window_size * self.sample_rate;
                let grain_offset = abs_window_size / 2.0 * pos;

                let mut read_pos = (buffer.write_head as f32 - offset) + grain_offset;

                if read_pos < 0.0 {
                    read_pos += buffer_size;
                }

                let index = read_pos as usize % buffer.data.len();

                let left_gain = 0.5 * (1.0 - stereo_pos);
                let right_gain = 0.5 * (1.0 + stereo_pos);

                out.0 += buffer.data[index].clone().0 * *gain * left_gain;
                out.1 += buffer.data[index].clone().1 * *gain * right_gain;
            });
        }

        self.feedback_sample = feedback;
        //TBD DRY WET
        //*sample *= 0.0;
        *signal.0 += out.0;
        *signal.1 += out.1;
    }

    pub fn render(&mut self, samples: (&mut f32, &mut f32)) {
        self.write((samples.0, samples.1));
        self.read(samples);
    }
}
