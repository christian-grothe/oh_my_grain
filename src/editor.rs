use nih_plug::nih_error;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use triple_buffer::Output;
use std::sync::{Arc, Mutex};
use waveform::Waveform;

use crate::delay::DrawData;
use crate::GranularDelayParams;
mod waveform;

const RED: (u8, u8, u8) = (201, 104, 104);
const GREEN: (u8, u8, u8) = (165, 182, 141);

#[derive(Lens)]
struct Data {
    params: Arc<GranularDelayParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (700, 750))
}

pub(crate) fn create(
    params: Arc<GranularDelayParams>,
    editor_state: Arc<ViziaState>,
    draw_data: Arc<Mutex<Output<DrawData>>>,
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
            waveform(cx, draw_data.clone());
        });
    })
}

fn waveform(cx: &mut Context, draw_data: Arc<Mutex<Output<DrawData>>>) {
    HStack::new(cx, |cx| {
        Waveform::new(
            cx,
            draw_data,
            Data::params,
            |params| &params.playhead_a.distance,
            |params| &params.playhead_b.distance,
        );
    })
    .min_top(Pixels(30.0))
    .left(Pixels(15.0))
    .right(Pixels(15.0))
    .height(Pixels(100.0))
    .class("waveform");
}

fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, "Oh-My-Grain")
            .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
            .font_weight(FontWeightKeyword::Thin)
            .font_size(25.0);
    })
    .left(Pixels(15.0))
    .top(Pixels(10.0))
    .right(Pixels(15.0))
    .height(Pixels(50.0))
    .text_align(TextAlign::Right)
    .width(Stretch(1.0));
}

fn controlls(cx: &mut Context) {
    HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, "Playhead A")
                    .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                    .font_weight(FontWeightKeyword::Medium)
                    .font_size(15.0)
                    .height(Pixels(20.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Pixels(0.0))
                    .color(Color::rgb(RED.0, RED.1, RED.2));

                Label::new(cx, "Distance");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_a.distance)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Window Size");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_a.window_size)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Grain Length");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_a.grain_size)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Density");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_a.density)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Pitch");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_a.pitch)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Gain");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_a.gain)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Chaos");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_a.chaos)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
            })
            .height(Auto);

            VStack::new(cx, |cx| {
                Label::new(cx, "Playhead B")
                    .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                    .font_weight(FontWeightKeyword::Medium)
                    .font_size(15.0)
                    .height(Pixels(20.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Pixels(0.0))
                    .color(Color::rgb(GREEN.0, GREEN.1, GREEN.2));

                Label::new(cx, "Distance");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_b.distance)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Window Size");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_b.window_size)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Grain Length");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_b.grain_size)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Density");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_b.density)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Pitch");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_b.pitch)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Gain");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_b.gain)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Chaos");
                ParamSlider::new(cx, Data::params, |params| &params.playhead_b.chaos)
                    .bottom(Pixels(10.0))
                    .set_style(ParamSliderStyle::FromLeft);
            })
            .height(Auto);

        VStack::new(cx, |cx| {
            Label::new(cx, "Main")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(15.0)
                .height(Pixels(20.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Label::new(cx, "Feedback");
            ParamSlider::new(cx, Data::params, |params| &params.feedback)
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Color");
            ParamSlider::new(cx, Data::params, |params| &params.color)
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Dry");
            ParamSlider::new(cx, Data::params, |params| &params.dry)
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
            Label::new(cx, "Wet");
            ParamSlider::new(cx, Data::params, |params| &params.wet)
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
        })
        .height(Auto);
    })
    .left(Pixels(15.0))
    .right(Pixels(15.0))
    .height(Auto);
}
