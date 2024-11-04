mod delay;
use nih_plug::prelude::*;
use std::sync::Arc;
use nih_plug_vizia::ViziaState;

mod editor;

const PLAY_HEADS: usize = 2;
const GRAIN_NUM: usize = 256;

pub struct GranularDelay {
    params: Arc<GranularDelayParams>,
    delay: delay::Delay,
}

#[derive(Params)]
struct GranularDelayParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "densA"]
    pub density_a: FloatParam,
    #[id = "distanceA"]
    pub distance_a: FloatParam,
    #[id = "windowSizeA"]
    pub window_size_a: FloatParam,
    #[id = "grainSizeA"]
    pub grain_size_a: FloatParam,

    #[id = "densB"]
    pub density_b: FloatParam,
    #[id = "distanceB"]
    pub distance_b: FloatParam,
    #[id = "windowSizeB"]
    pub window_size_b: FloatParam,
    #[id = "grainSizeB"]
    pub grain_size_b: FloatParam,

    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "color"]
    pub color: FloatParam,
}

impl Default for GranularDelay {
    fn default() -> Self {
        Self {
            params: Arc::new(GranularDelayParams::default()),
            delay: delay::Delay::new(PLAY_HEADS, GRAIN_NUM),
        }
    }
}

impl Default for GranularDelayParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            density_a: FloatParam::new(
                "Density A",
                1.0,
                FloatRange::Linear {
                    min: 0.125,
                    max: 10.0,
                },
            ),
            distance_a: FloatParam::new(
                "Distance A",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            window_size_a: FloatParam::new(
                "Window Size A",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            grain_size_a: FloatParam::new(
                "Grain Size A",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            density_b: FloatParam::new(
                "Density B",
                1.0,
                FloatRange::Linear {
                    min: 0.125,
                    max: 10.0,
                },
            ),
            distance_b: FloatParam::new(
                "Distance B",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            window_size_b: FloatParam::new(
                "Window Size B",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            grain_size_b: FloatParam::new(
                "Grain Size B",
                0.5,
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
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.delay.init(
            10 * buffer_config.sample_rate as usize,
            buffer_config.sample_rate,
        );
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.delay.feedback = self.params.feedback.smoothed.next();
        self.delay.set_alpha(self.params.color.smoothed.next());

        self.delay
            .set_distance(0, self.params.distance_a.smoothed.next());
        self.delay
            .set_density(0, self.params.density_a.smoothed.next());
        self.delay
            .set_window_size(0, self.params.window_size_a.smoothed.next());
        self.delay
            .set_grain_size(0, self.params.grain_size_a.smoothed.next());
        self.delay
            .set_distance(1, self.params.distance_b.smoothed.next());
        self.delay
            .set_density(1, self.params.density_b.smoothed.next());
        self.delay
            .set_window_size(1, self.params.window_size_b.smoothed.next());
        self.delay
            .set_grain_size(1, self.params.grain_size_b.smoothed.next());

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
