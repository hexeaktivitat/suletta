use std::{char::MAX, sync::Arc};

use fundsp::hacker::*;
use nih_plug::prelude::*;

struct Synth {
    params: Arc<SynthParams>,
    audio: Box<dyn AudioUnit64 + Send + Sync>,
}

#[derive(Params)]
struct SynthParams {}

impl Default for Synth {
    fn default() -> Self {
        let audio_graph = noise() >> split::<U2>();
        Self {
            params: Arc::new(SynthParams::default()),
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send + Sync>,
        }
    }
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for Synth {
    const NAME: &'static str = "Synth";
    const VENDOR: &'static str = "hexeaktivitat";
    const URL: &'static str = "http://no.website.really/";
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

impl ClapPlugin for Synth {
    const CLAP_ID: &'static str = "Synth";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Synth description");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::Instrument, ClapFeature::Synthesizer];
}

impl Vst3Plugin for Synth {
    const VST3_CLASS_ID: [u8; 16] = *b"Hexe-Synth-0.0.0";

    const VST3_CATEGORIES: &'static str = "Instrument|Synthesizer";
}

nih_export_clap!(Synth);
nih_export_vst3!(Synth);
