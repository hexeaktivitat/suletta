pub mod param_ids;

use std::{sync::Arc, todo};

use nih_plug::prelude::*;

use param_ids::*;

pub const MAX_BUFFER_SIZE: usize = 64;

struct Suletta {
    params: Arc<SulettaParams>,
    sample_rate: f32,
}

#[derive(Params)]
struct SulettaParams {
    #[id = "cutoff"]
    pub filter1_cutoff: FloatParam,
    #[id = "resonance"]
    pub filter1_resonance: FloatParam,
    #[id = "attack"]
    pub env1_attack: FloatParam,
    #[id = "decay"]
    pub env1_decay: FloatParam,
    #[id = "sustain"]
    pub env1_sustain: FloatParam,
    #[id = "release"]
    pub env1_release: FloatParam,
}

impl Default for Suletta {
    fn default() -> Self {
        let def_params = Arc::new(SulettaParams::default());

        Self {
            params: def_params,
            sample_rate: 41000f32,
        }
    }
}

impl Default for SulettaParams {
    fn default() -> Self {
        Self {
            filter1_cutoff: FloatParam::new(
                "Filter Cutoff",
                10000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Exponential(50.0))
            .with_unit(" Hz"),
            filter1_resonance: FloatParam::new(
                "Filter Resonance",
                1.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 10.0,
                },
            ),
            env1_attack: FloatParam::new(
                "Attack",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0)),
            env1_decay: FloatParam::new(
                "Decay",
                1.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0)),
            env1_sustain: FloatParam::new(
                "Sustain",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(1.0)),
            env1_release: FloatParam::new(
                "Release",
                1.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(1.0)),
        }
    }
}

impl Plugin for Suletta {
    const NAME: &'static str = "Suletta";
    const VENDOR: &'static str = "hexeaktivitat";
    const URL: &'static str = "https://github.com/hexeaktivitat/suletta";
    const EMAIL: &'static str = "hexeaktivitat@gmail.com";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        true
    }

    fn reset(&mut self) {
        unimplemented!();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for (_block_id, block) in buffer.iter_blocks(MAX_BUFFER_SIZE) {
            let mut block_channels = block.into_iter();
            let left_channel = block_channels.next().unwrap();
            let right_channel = block_channels.next().unwrap();

            MAX_BUFFER_SIZE;

            while let Some(event) = context.next_event() {
                match event {
                    NoteEvent::NoteOn { note, .. } => {
                        todo!();
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        todo!();
                    }
                    _ => (),
                }
            }

            let mut left_buf = [0f64; MAX_BUFFER_SIZE];
            let mut right_buf = [0f64; MAX_BUFFER_SIZE];

            for (chunk, output) in left_channel.iter_mut().zip(left_buf.iter()) {
                *chunk = *output as f32;
            }
            for (chunk, output) in right_channel.iter_mut().zip(right_buf.iter()) {
                *chunk = *output as f32;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Suletta {
    const CLAP_ID: &'static str = "Suletta";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Suletta description");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::Instrument, ClapFeature::Synthesizer];
}

impl Vst3Plugin for Suletta {
    const VST3_CLASS_ID: [u8; 16] = *b"SulettaSynthxxxx";

    const VST3_CATEGORIES: &'static str = "Instrument|Synthesizer";
}

nih_export_clap!(Suletta);
nih_export_vst3!(Suletta);
