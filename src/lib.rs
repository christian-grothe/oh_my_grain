mod delay;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;

pub struct GranularDelay {
    params: Arc<GranularDelayParams>,
    delay: delay::Delay,
}

#[derive(Params)]
struct GranularDelayParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,
    #[id = "distance_a"]
    pub distance_a: FloatParam,

    #[id = "distance_b"]
    pub distance_b: FloatParam,

    #[id = "distance_c"]
    pub distance_c: FloatParam,

    #[id = "distance_d"]
    pub distance_d: FloatParam,

    #[id = "feedback"]
    pub feedback: FloatParam,

    #[id = "color"]
    pub color: FloatParam,
}

impl Default for GranularDelay {
    fn default() -> Self {
        Self {
            params: Arc::new(GranularDelayParams::default()),
            delay: delay::Delay::new(44100 * 5),
        }
    }
}

impl Default for GranularDelayParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(250, 400),

            distance_a: FloatParam::new(
                "Distance_A",
                0.1,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            distance_b: FloatParam::new(
                "Distance_B",
                0.2,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            distance_c: FloatParam::new(
                "Distance_C",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            distance_d: FloatParam::new(
                "Distance_D",
                0.4,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            feedback: FloatParam::new("Feedback", 0.2, FloatRange::Linear { min: 0.0, max: 1.0 }),

            color: FloatParam::new("Color", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

impl Plugin for GranularDelay {
    const NAME: &'static str = "Granular Delay";
    const VENDOR: &'static str = "Christian Grothe";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "christian.grothe@posteo.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.label("Granular Delay");
                    ui.label("DistA");
                    ui.add(widgets::ParamSlider::for_param(&params.distance_a, setter));
                    ui.label("DistB");
                    ui.add(widgets::ParamSlider::for_param(&params.distance_b, setter));
                    ui.label("DistC");
                    ui.add(widgets::ParamSlider::for_param(&params.distance_c, setter));
                    ui.label("DistD");
                    ui.add(widgets::ParamSlider::for_param(&params.distance_d, setter));
                    ui.label("Feedback");
                    ui.add(widgets::ParamSlider::for_param(&params.feedback, setter));
                    ui.label("Color");
                    ui.add(widgets::ParamSlider::for_param(&params.color, setter));
                });
            },
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            self.delay
                .set_distance(0, self.params.distance_a.smoothed.next());
            // self.delay
            //     .set_distance(1, self.params.distance_b.smoothed.next());
            // self.delay
            //     .set_distance(2, self.params.distance_c.smoothed.next());
            // self.delay
            //     .set_distance(3, self.params.distance_d.smoothed.next());

            self.delay.feedback = self.params.feedback.smoothed.next();
            self.delay.set_alpha(self.params.color.smoothed.next());

            for sample in channel_samples {
                self.delay.render(sample)
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for GranularDelay {
    const CLAP_ID: &'static str = "com.christian-grothe.granular-delay";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple granular delay");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for GranularDelay {
    const VST3_CLASS_ID: [u8; 16] = *b"_granular_delay_";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(GranularDelay);
nih_export_vst3!(GranularDelay);
