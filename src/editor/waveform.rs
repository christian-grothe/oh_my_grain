use std::sync::{Arc, RwLock};

use nih_plug::params::Param;
use nih_plug_vizia::{
    vizia::{
        binding::Lens,
        context::{Context, DrawContext},
        vg::{Color, Paint, Path, Solidity},
        view::{Canvas, Handle, View},
    },
    widgets::param_base::ParamWidgetBase,
};

use crate::delay::{Buffer, Graindata};

const RED: (u8, u8, u8) = (201, 104, 104);
const GREEN: (u8, u8, u8) = (165, 182, 141);

pub struct Waveform {
    dist_a_param: ParamWidgetBase,
    dist_b_param: ParamWidgetBase,
    buffer: Arc<RwLock<Buffer>>,
    draw_data: Arc<RwLock<Vec<Graindata>>>,
}

impl Waveform {
    pub fn new<L, Params, P, AMap, BMap>(
        cx: &mut Context,
        buffer: Arc<RwLock<Buffer>>,
        draw_data: Arc<RwLock<Vec<Graindata>>>,
        params: L,
        params_to_param_dist_a: AMap,
        params_to_param_dist_b: BMap,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        AMap: Fn(&Params) -> &P + Copy + 'static,
        BMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            dist_a_param: ParamWidgetBase::new(cx, params, params_to_param_dist_a),
            dist_b_param: ParamWidgetBase::new(cx, params, params_to_param_dist_b),
            buffer,
            draw_data,
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

        let paint = Paint::color(Color::rgb(200, 200, 200));
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
            let mut bar_height = (*l + *r) / chunks as f32;

            if bar_height > 1.0 {
                bar_height = 1.0;
            }

            path.rect(
                bounds.x + bounds.w * i as f32 / buffer_to_draw.len() as f32,
                (bounds.y + bounds.h / 2.0) - (bounds.h * bar_height / 2.0),
                2.0,
                bounds.h * bar_height,
            );
        }

        canvas.fill_path(&path, &paint);

        let mut path = Path::new();
        let paint = Paint::color(Color::rgb(RED.0, RED.1, RED.2));
        path.rect(
            bounds.x + bounds.w * (1.0 - self.dist_a_param.unmodulated_normalized_value()) - 2.5,
            bounds.y,
            5.0,
            bounds.h,
        );

        canvas.fill_path(&path, &paint);

        let mut path = Path::new();
        let paint = Paint::color(Color::rgb(GREEN.0, GREEN.1, GREEN.2));
        path.rect(
            bounds.x + bounds.w * (1.0 - self.dist_b_param.unmodulated_normalized_value()) - 2.5,
            bounds.y,
            5.0,
            bounds.h,
        );

        canvas.fill_path(&path, &paint);

        let draw_data = self.draw_data.read().unwrap();
        let paint = Paint::color(Color::hex("#F6EABE"));

        draw_data.iter().for_each(|data| {
            let mut path = Path::new();
            let y = (data.stereo_pos + 1.0) / 2.0;
            path.arc(
                bounds.x + bounds.w * data.pos,
                bounds.y + bounds.h * y,
                1.0 + 5.0 * data.gain,
                0.0,
                2.0 * std::f32::consts::PI,
                Solidity::Hole,
            );
            canvas.stroke_path(&path, &paint);
        });
    }
}
