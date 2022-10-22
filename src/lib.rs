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
    env1_active: An<Var<f64>>,
    env1_finish: An<Var<f64>>,
}

#[derive(Params)]
struct SulettaParams {
    /*     #[id = "freq"]
    pub osc1_frequency: FloatParam,
    #[id = "amp"]
    pub osc1_amp: FloatParam, */
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

        let atk = def_params.env1_attack.plain_value().to_f64();
        let dcy = def_params.env1_decay.plain_value().to_f64();
        let sus = def_params.env1_sustain.plain_value().to_f64();
        let rel = def_params.env1_release.plain_value().to_f64();
        let active = var(ENV1_ACTIVE, 0.0);
        let finished = var(ENV1_FINISH, 1.0);

        let env = || adsr_live(atk, dcy, sus, rel, active, finished);

        let audio_graph = frq()
            >> (env() * saw())
            >> (pass() | filt_cut() | reso())
            >> lowpass()
            >> declick()
            >> split::<U2>();

        // let audio_graph = frq() >> sine() * frq() * modulate() + frq() >> sine() >> split::<U2>();

        // let audio_graph = sine_hz(440.) * 440. * 1. + 440. >> sine() >> split::<U2>();
        Self {
            params: def_params,
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send + Sync>,
            midi_note_id: 0,
            midi_note_freq: midi_freq,
            midi_note_gain: Smoother::new(SmoothingStyle::Linear(5.0)),
            sample_rate: 41000f32,
            time: Duration::default(),
            enabled: false,
            env1_active: var(ENV1_ACTIVE, 0.0),
            env1_finish: var(ENV1_FINISH, 1.0),
        }
    }
}

impl Default for SulettaParams {
    fn default() -> Self {
        Self {
            /* osc1_frequency: FloatParam::new(
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
            .with_unit(" dB"), */
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
                    max: 100.0,
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
                    max: 100.0,
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
            let left_channel = block_channels.next().unwrap();
            let right_channel = block_channels.next().unwrap();

            while let Some(event) = context.next_event() {
                match event {
                    NoteEvent::NoteOn { note, .. } => {
                        self.env1_active.set(ENV1_ACTIVE, 1.0);
                        self.midi_note_id = note;
                        self.midi_note_freq = util::midi_note_to_freq(note);
                        self.audio.set(MIDI_ON, self.time.as_secs_f64());
                        self.enabled = true;
                        self.audio.reset(Some(self.sample_rate.to_f64()));
                        self.env1_active.set(ENV1_ACTIVE, 0.0);
                    }
                    NoteEvent::NoteOff { note, .. } if note == self.midi_note_id => {
                        self.time = Duration::default();
                        self.enabled = false;
                        self.env1_active.set(ENV1_ACTIVE, 1.0) // send release code
                    }
                    _ => (),
                }
            }

            if self.env1_finish.value() > 0.0 {
                self.env1_finish.set(ENV1_FINISH, 0.0);
                self.env1_active.set(ENV1_ACTIVE, 0.0);
                self.midi_note_freq = 0.0;
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
                //self.time += Duration::from_secs_f32(MAX_BUFFER_SIZE as f32 / self.sample_rate);

                self.audio
                    .process(MAX_BUFFER_SIZE, &[], &mut [&mut left_buf, &mut right_buf]);
            }

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
