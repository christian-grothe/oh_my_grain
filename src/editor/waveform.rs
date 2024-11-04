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

pub struct Waveform {
    dist_a_param: ParamWidgetBase,
}

impl Waveform {
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param_dist_a: FMap,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            dist_a_param: ParamWidgetBase::new(cx, params, params_to_param_dist_a),
        }
        .build(
            cx,
            // This is an otherwise empty element only used for custom drawing
            |_cx| (),
        )
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

        let rect_paint = Paint::color(Color::rgb(20, 39, 123));
        let mut path = Path::new();
        // path.rect(bounds.x, bounds.y + bounds.h / 2.0, bounds.w, 10.0);
        path.rect(
            bounds.x + bounds.w * self.dist_a_param.unmodulated_normalized_value(),
            bounds.y,
            20.0, 
            bounds.h,
        );
        canvas.fill_path(&path, &rect_paint);
    }
}
