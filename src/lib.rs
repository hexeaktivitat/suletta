pub mod param_ids;

use std::{sync::Arc, time::Duration};

use fundsp::hacker::*;
use nih_plug::prelude::*;

use param_ids::*;

struct Suletta {
    params: Arc<SulettaParams>,
    audio: Box<dyn AudioUnit64 + Send + Sync>,
    //graph: Net64,
    midi_note_id: u8,
    midi_note_freq: f32,
    midi_note_gain: Smoother<f32>,
    sample_rate: f32,
    time: Duration,
    enabled: bool,
}

#[derive(Params)]
struct SulettaParams {
    #[id = "freq"]
    pub osc1_frequency: FloatParam,
    #[id = "amp"]
    pub osc1_amp: FloatParam,
    #[id = "cutoff"]
    pub filter1_cutoff: FloatParam,
    #[id = "resonance"]
    pub filter1_resonance: FloatParam,
}

impl Default for Suletta {
    fn default() -> Self {
        let def_params = Arc::new(SulettaParams::default());
        let midi_freq: f32 = 0.0;

        //let frq = || tag(OSC1_FREQ, def_params.osc1_frequency.plain_value().to_f64());
        let frq = || tag(OSC1_FREQ, midi_freq as f64);

        let filt_cut = || {
            tag(
                FILT1_CUTOFF,
                def_params.filter1_cutoff.plain_value().to_f64(),
            )
        };
        let reso = || tag(FILT1_RESO, def_params.filter1_cutoff.plain_value().to_f64());

        let offset = || tag(MIDI_ON, 0.0);
        let env = || offset() >> envelope2(|t, offset| downarc((t - offset) * 2.0));

        let audio_graph = frq()
            >> (env() * saw())
            >> (pass() | filt_cut() | reso())
            >> lowpass()
            >> declick()
            // >> lowpass_hz(10000.0, 0.0)
	        >> split::<U2>();

        // let audio_graph = frq() >> sine() * frq() * modulate() + frq() >> sine() >> split::<U2>();

        // let audio_graph = sine_hz(440.) * 440. * 1. + 440. >> sine() >> split::<U2>();
        Self {
            params: def_params,
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send + Sync>,
            midi_note_id: 0,
            midi_note_freq: midi_freq,
            midi_note_gain: Smoother::new(SmoothingStyle::Linear(5.0)),
            sample_rate: 1.0,
            time: Duration::default(),
            enabled: false,
        }
    }
}

impl Default for SulettaParams {
    fn default() -> Self {
        Self {
            osc1_frequency: FloatParam::new(
                "Frequency",
                440.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Exponential(50.0))
            .with_unit(" Hz")
            .hide(),
            osc1_amp: FloatParam::new(
                "Osc1 Vol",
                -10.0,
                FloatRange::Linear {
                    min: -30.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(3.0))
            .with_step_size(0.01)
            .with_unit(" dB"),
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
        _context: &mut impl InitContext,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        true
    }

    fn reset(&mut self) {
        self.midi_note_freq = 0.0;
        self.midi_note_id = 0;
        self.midi_note_gain.reset(0.0);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        for (_block_id, block) in buffer.iter_blocks(MAX_BUFFER_SIZE) {
            let mut block_channels = block.into_iter();
            let stereo_slice = &mut [
                block_channels.next().unwrap(),
                block_channels.next().unwrap(),
            ];

            while let Some(event) = context.next_event() {
                match event {
                    NoteEvent::NoteOn { note, .. } => {
                        self.midi_note_id = note;
                        self.midi_note_freq = util::midi_note_to_freq(note);
                        self.audio.set(MIDI_ON, self.time.as_secs_f64());
                        self.enabled = true;
                    }
                    NoteEvent::NoteOff { note, .. } if note == self.midi_note_id => {
                        self.midi_note_freq = 0.0;
                    }
                    _ => (),
                }
            }

            let mut left_buf = [0f64; MAX_BUFFER_SIZE];
            let mut right_buf = [0f64; MAX_BUFFER_SIZE];

            self.audio.set(OSC1_FREQ, self.midi_note_freq as f64);
            self.audio.set(
                FILT1_CUTOFF,
                self.params.filter1_cutoff.plain_value().to_f64(),
            );
            self.audio.set(
                FILT1_RESO,
                self.params.filter1_resonance.plain_value().to_f64(),
            );

            if self.enabled {
                self.time += Duration::from_secs_f32(MAX_BUFFER_SIZE as f32 / self.sample_rate);

                self.audio
                    .process(MAX_BUFFER_SIZE, &[], &mut [&mut left_buf, &mut right_buf]);
            }
            for (chunk, output) in stereo_slice[0].iter_mut().zip(left_buf.iter()) {
                *chunk = *output as f32;
            }
            for (chunk, output) in stereo_slice[1].iter_mut().zip(right_buf.iter()) {
                *chunk = *output as f32;
            }
        }

        ProcessStatus::KeepAlive
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
