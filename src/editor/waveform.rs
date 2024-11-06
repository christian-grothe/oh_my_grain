use std::sync::{Arc, RwLock};

use nih_plug::params::Param;
use nih_plug_vizia::{
    vizia::{
        binding::Lens,
        context::{Context, DrawContext},
        vg::{Color, Paint, Path},
        view::{Canvas, Handle, View},
    },
    widgets::param_base::ParamWidgetBase,
};

use crate::delay::Buffer;

pub struct Waveform {
    dist_a_param: ParamWidgetBase,
    window_size_a_param: ParamWidgetBase,
    dist_b_param: ParamWidgetBase,
    window_size_b_param: ParamWidgetBase,
    buffer: Arc<RwLock<Buffer>>,
}

impl Waveform {
    pub fn new<L, Params, P, DAMap, WAMap, DBMap, WBMap>(
        cx: &mut Context,
        buffer: Arc<RwLock<Buffer>>,
        params: L,
        params_to_param_dist_a: DAMap,
        params_to_param_window_size_a: WAMap,
        params_to_param_dist_b: DBMap,
        params_to_param_window_size_b: WBMap,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        DAMap: Fn(&Params) -> &P + Copy + 'static,
        WAMap: Fn(&Params) -> &P + Copy + 'static,
        DBMap: Fn(&Params) -> &P + Copy + 'static,
        WBMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            dist_a_param: ParamWidgetBase::new(cx, params, params_to_param_dist_a),
            window_size_a_param: ParamWidgetBase::new(cx, params, params_to_param_window_size_a),
            dist_b_param: ParamWidgetBase::new(cx, params, params_to_param_dist_b),
            window_size_b_param: ParamWidgetBase::new(cx, params, params_to_param_window_size_b),
            buffer,
        }
        .build(cx, |_cx| ())
    }
}

impl View for Waveform {
    fn element(&self) -> Option<&'static str> {
        Some("waveform")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let paint = Paint::color(Color::rgb(100, 100, 100));
        let mut path = Path::new();
        let buffer = self.buffer.read().unwrap();
        let chunks = buffer.data.len() / 128;

        let buffer_to_draw: Vec<(f32, f32)> = buffer.data[buffer.write_head..]
            .iter()
            .chain(buffer.data[..buffer.write_head].iter())
            .collect::<Vec<&(f32, f32)>>()
            .chunks(chunks)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold((0.0, 0.0), |acc, (x, y)| (acc.0 + x.abs(), acc.1 + y.abs()))
            })
            .collect();

        for (i, (l, r)) in buffer_to_draw.iter().enumerate() {
            let bar_height = (*l + *r) / chunks as f32;
            path.rect(
                bounds.x + bounds.w * i as f32 / buffer_to_draw.len() as f32,
                (bounds.y + bounds.h / 2.0) - (bounds.h * bar_height / 2.0),
                2.0,
                bounds.h * bar_height,
            );
        }

        canvas.fill_path(&path, &paint);

        let mut path = Path::new();
        let paint = Paint::color(Color::rgb(200, 100, 100));
        path.rect(
            bounds.x + bounds.w * (1.0 - self.dist_a_param.unmodulated_normalized_value()),
            bounds.y,
            50.0 * self.window_size_a_param.unmodulated_normalized_value(),
            bounds.h,
        );

        canvas.stroke_path(&path, &paint);

        let mut path = Path::new();
        let paint = Paint::color(Color::rgb(100, 200, 100));
        path.rect(
            bounds.x + bounds.w * (1.0 - self.dist_b_param.unmodulated_normalized_value()),
            bounds.y,
            50.0 * self.window_size_b_param.unmodulated_normalized_value(),
            bounds.h,
        );

        canvas.stroke_path(&path, &paint);
    }
}
