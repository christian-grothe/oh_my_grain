use nih_plug::nih_error;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use waveform::Waveform;

use crate::GranularDelayParams;
mod waveform;

#[derive(Lens)]
struct Data {
    params: Arc<GranularDelayParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (600, 400))
}

pub(crate) fn create(
    params: Arc<GranularDelayParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        if let Err(err) = cx.add_stylesheet(include_style!("src/editor/styles.css")) {
            nih_error!("Failed to load stylesheet: {err:?}")
        }

        Data {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            top_bar(cx);
            controlls(cx);
            waveform(cx);
        });
    })
}

fn waveform(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Waveform::new(
            cx,
            Data::params,
            |params| &params.distance_a,
        );
    })
    .height(Pixels(100.0))
    .class("section");
}

fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, "Granular Delay")
            .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
            .font_weight(FontWeightKeyword::Thin)
            .font_size(37.0)
            .top(Pixels(2.0))
            .left(Pixels(8.0));
    })
    .height(Pixels(50.0));
}

fn controlls(cx: &mut Context) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            Label::new(cx, "Playhead A")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(15.0)
                .height(Pixels(20.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Label::new(cx, "Distance");
            ParamSlider::new(cx, Data::params, |params| &params.distance_a)
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Window Size");
            ParamSlider::new(cx, Data::params, |params| &params.window_size_a)
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Grain Size");
            ParamSlider::new(cx, Data::params, |params| &params.grain_size_a)
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Density");
            ParamSlider::new(cx, Data::params, |params| &params.density_a)
                .set_style(ParamSliderStyle::FromLeft);
        })
        .height(Auto);

        VStack::new(cx, |cx| {
            Label::new(cx, "Playhead B")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(15.0)
                .height(Pixels(20.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Label::new(cx, "Distance");
            ParamSlider::new(cx, Data::params, |params| &params.distance_b)
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Window Size");
            ParamSlider::new(cx, Data::params, |params| &params.window_size_b)
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Grain Size");
            ParamSlider::new(cx, Data::params, |params| &params.grain_size_b)
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Density");
            ParamSlider::new(cx, Data::params, |params| &params.density_b)
                .set_style(ParamSliderStyle::FromLeft);
        })
        .height(Auto);
    })
    .height(Auto);
}
