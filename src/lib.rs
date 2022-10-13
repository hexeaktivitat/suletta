use std::sync::Arc;

use fundsp::hacker::*;
use nih_plug::prelude::*;

struct Suletta {
    params: Arc<SulettaParams>,
    audio: Box<dyn AudioUnit64 + Send + Sync>,
    //graph: Net64,
}

#[derive(Params)]
struct SulettaParams {
    #[id = "freq"]
    pub frequency: FloatParam,
    #[id = "modulation"]
    pub modulation: FloatParam,
    #[id = "cutoff"]
    pub filter_cutoff: FloatParam,
    #[id = "resonance"]
    pub filter_resonance: FloatParam,
}

impl Default for Suletta {
    fn default() -> Self {
        let def_params = Arc::new(SulettaParams::default());

        let frq = || tag(0, def_params.frequency.plain_value().to_f64());
        let modulate = || tag(1, def_params.modulation.plain_value().to_f64());
	let filt_cut = || tag(2, def_params.filter_cutoff.plain_value().to_f64());
	let reso = || tag(3, def_params.filter_cutoff.plain_value().to_f64());

        let audio_graph = (frq() >> saw()) >> lowpass_hz(1000.0, 1.0) >> split::<U2>();

	// let audio_graph = frq() >> sine() * frq() * modulate() + frq() >> sine() >> split::<U2>();

	
        // let audio_graph = sine_hz(440.) * 440. * 1. + 440. >> sine() >> split::<U2>();
        Self {
            params: def_params,
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send + Sync>,
            //graph: Net64::new(0,2),
        }
    }
}

impl Default for SulettaParams {
    fn default() -> Self {
        Self {
            frequency: FloatParam::new(
                "Frequency",
                440.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            ),
            modulation: FloatParam::new(
                "Modulation",
                1.0,
                FloatRange::Linear {
		    min: 1.0,
		    max: 5.0
		},
            ),
            filter_cutoff: FloatParam::new(
                "Filter Cutoff",
                10000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            ),
	    filter_resonance: FloatParam::new(
		"Filter Resonance",
		0.0,
		FloatRange::Linear {
		    min: 0.0,
		    max: 100.0
		}
	    ),
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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext,
    ) -> bool {
        true
    }

    fn reset(&mut self) {
        // this space intentionally left blank
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        for (_, block) in buffer.iter_blocks(MAX_BUFFER_SIZE) {
            let mut block_channels = block.into_iter();
            let stereo_slice = &mut [
                block_channels.next().unwrap(),
                block_channels.next().unwrap(),
            ];

            let mut left_buf = [0f64; MAX_BUFFER_SIZE];
            let mut right_buf = [0f64; MAX_BUFFER_SIZE];

            self.audio
                .set(0, self.params.frequency.plain_value().to_f64());
            self.audio
                .set(1, self.params.modulation.plain_value().to_f64());

            self.audio
                .process(MAX_BUFFER_SIZE, &[], &mut [&mut left_buf, &mut right_buf]);

            for (chunk, output) in stereo_slice[0].iter_mut().zip(left_buf.iter()) {
                *chunk = *output as f32;
            }
            for (chunk, output) in stereo_slice[1].iter_mut().zip(right_buf.iter()) {
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
    const VST3_CLASS_ID: [u8; 16] = *b"Hexe-Synth-0.0.0";

    const VST3_CATEGORIES: &'static str = "Instrument|Synthesizer";
}

nih_export_clap!(Suletta);
nih_export_vst3!(Suletta);
