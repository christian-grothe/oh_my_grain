use std::sync::{Arc, RwLock};

mod envelope;
mod filter;
mod playhead;

pub struct Buffer {
    pub data: Vec<(f32, f32)>,
    pub write_head: usize,
}

impl Buffer {
    fn get_cubic_sample(&self, pos: f32) -> (f32, f32) {
        let len = self.data.len();

        let base = pos.floor() as usize;

        let t = pos - pos.floor();

        let p0 = self.data[(base.wrapping_sub(1)) % len];
        let p1 = self.data[base % len];
        let p2 = self.data[(base + 1) % len];
        let p3 = self.data[(base + 2) % len];

        let left = p1.0
            + 0.5
                * t
                * (p2.0 - p0.0
                    + t * (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0
                        + t * (3.0 * (p1.0 - p2.0) + p3.0 - p0.0)));
        let right = p1.1
            + 0.5
                * t
                * (p2.1 - p0.1
                    + t * (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1
                        + t * (3.0 * (p1.1 - p2.1) + p3.1 - p0.1)));
        (left, right)
    }
}

#[derive(Debug)]
pub struct Graindata {
    pub pos: f32,
    pub stereo_pos: f32,
    pub gain: f32,
}

pub struct Delay {
    pub buffer: Arc<RwLock<Buffer>>,
    pub draw_data: Arc<RwLock<Vec<Graindata>>>,
    draw_data_update_count: usize,
    pub sample_rate: f32,
    play_heads: Vec<playhead::PlayHead>,
    pub feedback: f32,
    filter: filter::StereoBiquadLowPass,
    feedback_sample: (f32, f32),
    dry: f32,
    wet: f32,
}

impl Delay {
    pub fn new(play_heads: usize, grain_num: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Buffer {
                data: vec![(0.0, 0.0); 1024],
                write_head: 0,
            })),
            draw_data: Arc::new(RwLock::new(vec![Graindata {
                pos: 0.0,
                stereo_pos: 0.0,
                gain: 0.0,
            }])),
            draw_data_update_count: 0,
            sample_rate: 0.0,
            feedback: 0.0,
            play_heads: (0..play_heads)
                .map(|_| playhead::PlayHead::new(0.5, grain_num))
                .collect(),
            filter: filter::StereoBiquadLowPass::new(5000.0, 48_000.0, 0.707),
            feedback_sample: (0.0, 0.0),
            dry: 1.0,
            wet: 1.0,
        }
    }

    pub fn init(&mut self, buffer_length_sec: f32, sample_rate: f32) {
        let mut buffer = self.buffer.write().unwrap();
        let buffer_length = (buffer_length_sec * sample_rate) as usize;

        buffer.data.resize(buffer_length, (0.0, 0.0));
        self.filter.update_sample_rate(sample_rate);

        self.sample_rate = sample_rate;
        self.play_heads.iter_mut().for_each(|play_head| {
            play_head.init(sample_rate, buffer_length_sec);
        });
    }

    pub fn set_gain(&mut self, index: usize, value: f32) {
        self.play_heads[index].set_gain(value);
    }

    pub fn set_pitch(&mut self, index: usize, value: i32) {
        self.play_heads[index].set_pitch(value);
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

    pub fn set_cutoff(&mut self, value: f32) {
        let cutoff = value * 10_000.0 + 100.0;
        self.filter.update_coefficients(cutoff);
    }

    pub fn set_dry(&mut self, value: f32) {
        self.dry = value;
    }

    pub fn set_wet(&mut self, value: f32) {
        self.wet = value;
    }

    pub fn get_draw_data(&mut self) {
        self.draw_data_update_count += 1;
        if self.draw_data_update_count >= self.sample_rate as usize / 60 {
            let mut draw_data = self.draw_data.write().unwrap();
            draw_data.clear();
            self.play_heads.iter().for_each(|play_head| {
                play_head.grains.iter().for_each(|grain| {
                    if grain.active {
                        draw_data.push(Graindata {
                            pos: 1.0 - grain.pos,
                            stereo_pos: grain.stereo_pos,
                            gain: grain.gain,
                        })
                    }
                })
            });

            self.draw_data_update_count = 0;
        }
    }

    fn write(&mut self, signal: (&f32, &f32)) {
        let feedback = (
            self.feedback_sample.0 * self.feedback * 0.5,
            self.feedback_sample.1 * self.feedback * 0.5,
        );

        let feedback = self.filter.process(feedback);
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

            if play_head.feedback_src == playhead::FeedbackSrc::Playhead {
                let offset = buffer_size * play_head.current_distance;

                let mut feedback_pos = buffer.write_head as f32 - offset;

                if feedback_pos < 0.0 {
                    feedback_pos += buffer_size;
                }

                feedback.0 += buffer.data[feedback_pos as usize % buffer.data.len()].0;
                feedback.1 += buffer.data[feedback_pos as usize % buffer.data.len()].1;
            }

            let grain_data = play_head.get_grain_data();

            grain_data.iter().for_each(|(pos, gain, stereo_pos)| {
                let offset = buffer_size * pos;

                let mut read_pos = buffer.write_head as f32 - offset;

                if read_pos < 0.0 {
                    read_pos += buffer_size;
                }

                let left_gain = 0.5 * (1.0 - stereo_pos);
                let right_gain = 0.5 * (1.0 + stereo_pos);

                let (left_sample, right_sample) = buffer.get_cubic_sample(read_pos);

                // this seems not to work so just using FeedbackSrc::Playhead for now
                if play_head.feedback_src == playhead::FeedbackSrc::Grain {
                    feedback.0 += left_sample;
                    feedback.1 += right_sample;
                }

                out.0 += left_sample * *gain * left_gain;
                out.1 += right_sample * *gain * right_gain;
            });
        }

        self.feedback_sample = feedback;

        *signal.0 *= self.dry;
        *signal.1 *= self.dry;

        *signal.0 += out.0 * self.wet;
        *signal.1 += out.1 * self.wet;
    }

    pub fn render(&mut self, samples: (&mut f32, &mut f32)) {
        self.get_draw_data();
        self.write((samples.0, samples.1));
        self.read(samples);
    }
}
