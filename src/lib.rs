mod delay;
use delay::DrawData;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::{Arc, Mutex};
use triple_buffer::{triple_buffer, Output};

mod editor;

pub struct GranularDelay {
    params: Arc<GranularDelayParams>,
    delay: delay::Delay,
    buf_output: Arc<Mutex<Output<DrawData>>>,
}

#[derive(Params)]
struct PlayheadParams {
    #[id = "dens"]
    pub density: FloatParam,
    #[id = "distance"]
    pub distance: FloatParam,
    #[id = "windowSize"]
    pub window_size: FloatParam,
    #[id = "grainSize"]
    pub grain_size: FloatParam,
    #[id = "pitch"]
    pub pitch: IntParam,
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "chaos"]
    pub chaos: FloatParam,
}

impl PlayheadParams {
    fn new(distance: f32) -> Self {
        PlayheadParams {
            density: FloatParam::new(
                "Density",
                1.0,
                FloatRange::Linear {
                    min: 0.125,
                    max: 50.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2)),

            distance: FloatParam::new(
                "Distance",
                distance,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),

            window_size: FloatParam::new(
                "Window Size",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),

            grain_size: FloatParam::new(
                "Grain Size",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),

            gain: FloatParam::new("Gain", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0)),

            chaos: FloatParam::new("Chaos", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0)),

            pitch: IntParam::new("Pitch", 0, IntRange::Linear { min: -12, max: 12 })
                .with_unit(" st"),
        }
    }
}

#[derive(Params)]
struct GranularDelayParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[nested(id_prefix = "a", group = "playheads")]
    playhead_a: PlayheadParams,
    #[nested(id_prefix = "b", group = "playheads")]
    playhead_b: PlayheadParams,

    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "color"]
    pub color: FloatParam,
    #[id = "dry"]
    pub dry: FloatParam,
    #[id = "wet"]
    pub wet: FloatParam,
}

impl Default for GranularDelay {
    fn default() -> Self {
        let (buf_input, buf_output) = triple_buffer(&DrawData::new());
        Self {
            params: Arc::new(GranularDelayParams::default()),
            delay: delay::Delay::new(buf_input),
            buf_output: Arc::new(Mutex::new(buf_output)),
        }
    }
}

impl Default for GranularDelayParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            playhead_a: PlayheadParams::new(0.25),
            playhead_b: PlayheadParams::new(0.5),

            feedback: FloatParam::new("Feedback", 0.45, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0)),

            color: FloatParam::new("Color", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2)),

            dry: FloatParam::new("Dry", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0)),

            wet: FloatParam::new("Wet", 0.85, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0)),
        }
    }
}

impl Plugin for GranularDelay {
    const NAME: &'static str = "Oh My Grain";
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
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.buf_output.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.delay.init(buffer_config.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.delay
            .set_distance(0, self.params.playhead_a.distance.smoothed.next());
        self.delay
            .set_density(0, self.params.playhead_a.density.smoothed.next());
        self.delay
            .set_window_size(0, self.params.playhead_a.window_size.smoothed.next());
        self.delay
            .set_grain_size(0, self.params.playhead_a.grain_size.smoothed.next());
        self.delay
            .set_pitch(0, self.params.playhead_a.pitch.smoothed.next());
        self.delay
            .set_gain(0, self.params.playhead_a.gain.smoothed.next());
        self.delay
            .set_pitch(0, self.params.playhead_a.pitch.smoothed.next());
        self.delay
            .set_gain(0, self.params.playhead_a.gain.smoothed.next());
        self.delay
            .set_chaos(0, self.params.playhead_a.chaos.smoothed.next());

        self.delay
            .set_distance(1, self.params.playhead_b.distance.smoothed.next());
        self.delay
            .set_density(1, self.params.playhead_b.density.smoothed.next());
        self.delay
            .set_window_size(1, self.params.playhead_b.window_size.smoothed.next());
        self.delay
            .set_grain_size(1, self.params.playhead_b.grain_size.smoothed.next());
        self.delay
            .set_pitch(1, self.params.playhead_b.pitch.smoothed.next());
        self.delay
            .set_gain(1, self.params.playhead_b.gain.smoothed.next());
        self.delay
            .set_pitch(1, self.params.playhead_b.pitch.smoothed.next());
        self.delay
            .set_gain(1, self.params.playhead_b.gain.smoothed.next());
        self.delay
            .set_chaos(1, self.params.playhead_b.chaos.smoothed.next());

        self.delay.set_dry(self.params.dry.smoothed.next());
        self.delay.set_wet(self.params.wet.smoothed.next());
        self.delay.feedback = self.params.feedback.smoothed.next();
        self.delay.set_cutoff(self.params.color.smoothed.next());

        for channels in buffer.iter_samples() {
            let mut sample_channels = channels.into_iter();
            let stereo_slice = (
                sample_channels.next().unwrap(),
                sample_channels.next().unwrap(),
            );
            self.delay.render(stereo_slice);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for GranularDelay {
    const CLAP_ID: &'static str = "com.christian-grothe.oh-my-grain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A granular delay");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for GranularDelay {
    const VST3_CLASS_ID: [u8; 16] = *b"_oh__my__grain__";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(GranularDelay);
nih_export_vst3!(GranularDelay);
