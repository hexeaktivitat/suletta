# suletta

Rust VST3/CLAP synthesizer plugin. Primarily a learning project, but intended to be a useful tool for sound design and production.

Built using the [nih-plug](https://github.com/robbert-vdh/nih-plug) framework using [fundsp](https://github.com/SamiPerttu/fundsp) for DSP utility.

## Building

Clone the repository and use `cargo xtask bundle suletta --release` to build the VST3 and CLAP plugin files.
```
git clone https://github.com/hexeaktivitat/suletta.git
cd suletta
cargo xtask bundle suletta --release
```

Plugin files will be located in `./suletta/target/bundled`.

## Todo

- [ ] Implement core subtractive synthesis logic
    - [ ] Oscillator graph
        - [x] Basic sawtooth functionality
        - [ ] Choose waveform
    - [ ] Filter graph
        - [x] Basic lowpass functionality
        - [ ] Choose filter type
    - [ ] Envelope generators
        - [ ] Basic linear functionality
        - [ ] Linear & exponential envelopes
    - [ ] MIDI input functionality
        - [x] Note on/off handling
        - [ ] CC handling
- [ ] Determine overall goal of project
    - Options:
        - Sequencing synth designed with ambient in mind
        - General purpose virtual analogue subtractive synth
        - Hybrid FM/Subtractive/Wavetable synth
    - Do want sequencing capabilities and a robust modulation matrix
    - Static vs modular signal graph