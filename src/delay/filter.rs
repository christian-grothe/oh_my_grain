pub struct BiquadLowPass {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
    sample_rate: f32,
    q: f32,
}

impl BiquadLowPass {
    pub fn new(cutoff: f32, sample_rate: f32, q: f32) -> Self {
        let mut filter = Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
            sample_rate,
            q,
        };
        filter.update_coefficients(cutoff);
        filter
    }

    pub fn update_coefficients(&mut self, cutoff: f32) {
        let omega = 2.0 * std::f32::consts::PI * cutoff / self.sample_rate;
        let alpha = omega.sin() / (2.0 * self.q);
        let cos_omega = omega.cos();

        self.b0 = (1.0 - cos_omega) / 2.0;
        self.b1 = 1.0 - cos_omega;
        self.b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        self.a1 = -2.0 * cos_omega / a0;
        self.a2 = (1.0 - alpha) / a0;

        // Normalize feedforward coefficients
        self.b0 /= a0;
        self.b1 /= a0;
        self.b2 /= a0;
    }

    pub fn update_sample_rate(&mut self, sample_rate:f32){
        self.sample_rate = sample_rate;
        self.update_coefficients(5000.0);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.z1 + self.b2 * self.z2
            - self.a1 * self.z1
            - self.a2 * self.z2;
        self.z2 = self.z1;
        self.z1 = output;
        output
    }
}

pub struct StereoBiquadLowPass {
    filters: (BiquadLowPass, BiquadLowPass),
}

impl StereoBiquadLowPass {
    pub fn new(cutoff: f32, sample_rate: f32, q: f32) -> Self {
        let filter_l = BiquadLowPass::new(cutoff, sample_rate, q);
        let filter_r = BiquadLowPass::new(cutoff, sample_rate, q);

        Self {
            filters: (filter_l, filter_r),
        }
    }

    pub fn update_coefficients(&mut self, cutoff: f32) {
        self.filters.0.update_coefficients(cutoff);
        self.filters.1.update_coefficients(cutoff);
    }

    pub fn update_sample_rate(&mut self, sample_rate: f32){
        self.filters.0.update_sample_rate(sample_rate);
        self.filters.1.update_sample_rate(sample_rate);
    }

    pub fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        (
            self.filters.0.process(input.0),
            self.filters.1.process(input.1),
        )
    }
}
