use triple_buffer::Input;

mod envelope;
mod filter;
mod playhead;

const BUFFER_SIZE_SEC: f32 = 5.0;
const GRAIN_NUM: usize = 128;
const PLAY_HEADS: usize = 2;
const BAR_NUM: usize = 10;

#[derive(Clone)]
pub struct DrawData {
    pub buffer: Vec<f32>,
    pub grains: Vec<Graindata>,
}

impl DrawData {
    pub fn new() -> Self {
        Self {
            buffer: vec![0.0; BAR_NUM],
            grains: vec![
                Graindata {
                    pos: 0.0,
                    stereo_pos: 0.0,
                    gain: 0.0,
                };
                GRAIN_NUM
            ],
        }
    }
}

#[derive(Clone)]
pub struct Graindata {
    pub pos: f32,
    pub stereo_pos: f32,
    pub gain: f32,
}

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

#[derive(Default)]
struct DrawBuffer {
    sample_sum: f32,
    sample_count: usize,
    samples_per_bar: usize,
    current_index: usize,
}

pub struct Delay {
    pub buffer: Buffer,
    pub draw_data: Input<DrawData>,
    pub sample_rate: f32,
    play_heads: Vec<playhead::PlayHead>,
    pub feedback: f32,
    filter: filter::StereoBiquadLowPass,
    feedback_sample: (f32, f32),
    dry: f32,
    wet: f32,
    draw_buffer: DrawBuffer,
}

impl Delay {
    pub fn new(draw_data: Input<DrawData>) -> Self {
        Self {
            buffer: Buffer {
                data: vec![(0.0, 0.0); 1024],
                write_head: 0,
            },
            draw_data,
            sample_rate: 0.0,
            feedback: 0.0,
            play_heads: (0..PLAY_HEADS)
                .map(|_| playhead::PlayHead::new(0.5, GRAIN_NUM))
                .collect(),
            filter: filter::StereoBiquadLowPass::new(5000.0, 48_000.0, 0.707),
            feedback_sample: (0.0, 0.0),
            dry: 1.0,
            wet: 1.0,
            draw_buffer: DrawBuffer::default(),
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        let buffer_length = (BUFFER_SIZE_SEC * sample_rate) as usize;

        self.buffer.data.resize(buffer_length, (0.0, 0.0));
        self.filter.update_sample_rate(sample_rate);

        self.sample_rate = sample_rate;
        self.play_heads.iter_mut().for_each(|play_head| {
            play_head.init(sample_rate, BUFFER_SIZE_SEC);
        });

        self.draw_buffer.samples_per_bar = buffer_length / BAR_NUM;
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

    pub fn get_draw_data(&mut self, sample: f32) {
        let draw_data = self.draw_data.input_buffer();

        // grains
        draw_data.grains.clear();
        self.play_heads.iter().for_each(|play_head| {
            play_head.grains.iter().for_each(|grain| {
                if grain.active {
                    draw_data.grains.push(Graindata {
                        pos: 1.0 - grain.pos,
                        stereo_pos: grain.stereo_pos,
                        gain: grain.gain,
                    })
                }
            })
        });

        // waveform
        self.draw_buffer.sample_count += 1;
        self.draw_buffer.sample_sum += sample.abs();
        if self.draw_buffer.sample_count >= self.draw_buffer.samples_per_bar {
            //let mut new_bar = self.draw_buffer.sample_sum / self.draw_buffer.samples_per_bar as f32;
            let mut new_bar = rand::random::<f32>();

            if new_bar > 1.0 {
                new_bar = 1.0
            }

            draw_data.buffer[self.draw_buffer.current_index] = new_bar;
            self.draw_buffer.current_index = self.draw_buffer.current_index + 1;
            if self.draw_buffer.current_index >= draw_data.buffer.len() {
                self.draw_buffer.current_index = 0;
            }

            self.draw_buffer.sample_sum = 0.0;
            self.draw_buffer.sample_count = 0;

            nih_plug::nih_log!("{:?}", draw_data.buffer);
        }

        self.draw_data.publish();
    }

    fn write(&mut self, signal: (&f32, &f32)) {
        let feedback = (
            self.feedback_sample.0 * self.feedback * 0.5,
            self.feedback_sample.1 * self.feedback * 0.5,
        );

        let feedback = self.filter.process(feedback);
        let write_head = self.buffer.write_head;

        let left = signal.0 + feedback.0;
        let right = signal.1 + feedback.1;

        self.buffer.data[write_head].0 = left;
        self.buffer.data[write_head].1 = right;

        self.buffer.write_head = (self.buffer.write_head + 1) % self.buffer.data.len();

        self.get_draw_data(left + right);
    }

    fn read(&mut self, signal: (&mut f32, &mut f32)) {
        let mut out = (0.0, 0.0);
        let mut feedback = (0.0, 0.0);
        for play_head in self.play_heads.iter_mut() {
            play_head.update();

            let buffer_size = self.buffer.data.len() as f32;

            if play_head.feedback_src == playhead::FeedbackSrc::Playhead {
                let offset = buffer_size * play_head.current_distance;

                let mut feedback_pos = self.buffer.write_head as f32 - offset;

                if feedback_pos < 0.0 {
                    feedback_pos += buffer_size;
                }

                feedback.0 += self.buffer.data[feedback_pos as usize % self.buffer.data.len()].0;
                feedback.1 += self.buffer.data[feedback_pos as usize % self.buffer.data.len()].1;
            }

            let grain_data = play_head.get_grain_data();

            grain_data.iter().for_each(|(pos, gain, stereo_pos)| {
                let offset = buffer_size * pos;

                let mut read_pos = self.buffer.write_head as f32 - offset;

                if read_pos < 0.0 {
                    read_pos += buffer_size;
                }

                let left_gain = 0.5 * (1.0 - stereo_pos);
                let right_gain = 0.5 * (1.0 + stereo_pos);

                let (left_sample, right_sample) = self.buffer.get_cubic_sample(read_pos);

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
        self.write((samples.0, samples.1));
        self.read(samples);
    }
}
