//! Support for maintaining the usefulness of a corpus over time.
//!
//! Wasmtime's fuzzing strategy in general is to use `wasm-smith` to generate
//! modules which interprets fuzz input from libFuzzer as a sort of "DNA". This
//! works to generate pretty interesting modules but falls down over time
//! because the DNA to generate the same module over time can change. This
//! means that maintaining a corpus for Wasmtime is not the most useful thing
//! in the world unfortunately and any historical discoveries of coverage need
//! to be rediscovered every time the DNA changes.
//!
//! To help with this the module here implements a scheme where Wasmtime's fuzz
//! inputs are highly likely to be of the form:
//!
//! ```text
//! [ ... wasm module ... ][ .. fuzz custom section .. ]
//! ```
//!
//! The `fuzz custom section` here contains the original fuzz input used to
//! generate the `wasm module`, and if the DNA hasn't changed then it should
//! still be possible to do that as well. The benefit of this format, though,
//! is that if the DNA is changed then the interpretation of the `fuzz custom
//! section` will change but the original `wasm module` will not. This enables
//! us to populate the corpus, ideally, with a set of interesting `wasm module`
//! entries.
//!
//! Over time the `fuzz custom section` will "bitrot" and will be no longer able
//! to generate the original `wasm module`. The main consequence of this is that
//! when the original test case is mutated the generated wasm module from the
//! mutation will be nothing alike from the original test case's wasm module.
//! This means libFuzzer will have to rediscover ways to mutate into
//! interesting modules, but we're no worse off than before hopefully.
//! Additionally this more easily opens the door to integrate `wasm-mutate` one
//! day into mutation here as well.
//!
//! Currently this is all supported via two methods:
//!
//! 1. A custom mutator is registered with libfuzzer. This means that all
//!    inputs generated by the mutator, so long as they fit, will be the
//!    "envelope" format of this module. This means that the corpus will
//!    hopefully naturally get populated with wasm files rather than random
//!    inputs. Note that this is not guaranteed to succeed since sometimes the
//!    buffer to store the fuzz input in the mutator is not big enough to store
//!    the final wasm module, in which case a non-enveloped wasm module is
//!    stored.
//!
//! 2. If the environment variable `WRITE_FUZZ_INPUT_TO is set then the fuzz
//!    input, in its envelope format, will be written to the specified file.
//!    This can be useful in case an input is in its binary form or if a
//!    preexisting corpus is being rewritten.

use std::borrow::Cow;

use arbitrary::{Arbitrary, Result, Unstructured};
use wasm_encoder::Section;

/// Helper macro for fuzz targets that are single-module fuzzers.
///
/// This combines the features of this module into one macro invocation to
/// generate the fuzz entry point and mutator in tandem.
#[macro_export]
macro_rules! single_module_fuzzer {
    ($execute:ident $generate:ident) => {
        libfuzzer_sys::fuzz_target!(|data: &[u8]| {
            $crate::init_fuzzing();
            drop($crate::single_module_fuzzer::execute(
                data, $execute, $generate,
            ));
        });

        libfuzzer_sys::fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
            $crate::single_module_fuzzer::mutate(
                data,
                size,
                max_size,
                $generate,
                libfuzzer_sys::fuzzer_mutate,
            )
        });
    };
}

/// Executes a "single module fuzzer" given the raw `input` from libfuzzer.
///
/// This will use the `input` to generate `T`, some configuration, which is
/// then used by `gen_module` to generate a WebAssembly module. The module is
/// then passed to `run` along with the configuration and remaining data that
/// can be used as fuzz input.
///
/// The main purpose of this function is to handle when `input` is actually a
/// WebAssembly module "envelope". If the `input` is a valid wasm module and
/// ends with a specific trailing custom section then the module generated by
/// `gen_module` is actually discarded. The purpose of this is to handle the
/// case where the input used to generate a module may change over time but
/// we're still interested in the historical coverage of the original wasm
/// module.
pub fn execute<'a, T, U>(
    input: &'a [u8],
    run: fn(&[u8], KnownValid, T, &mut Unstructured<'a>) -> Result<U>,
    gen_module: fn(&mut T, &mut Unstructured<'a>) -> Result<(Vec<u8>, KnownValid)>,
) -> Result<U>
where
    T: Arbitrary<'a>,
{
    let (fuzz_data, module_in_input) = match extract_fuzz_input(input) {
        Ok(input) => {
            log::debug!("fuzz input was a valid module with trailing custom section");
            (input.fuzz_data, Some(input.module))
        }
        Err(e) => {
            log::debug!("fuzz input not a valid module: {e:?}");
            (input, None)
        }
    };
    let mut u = Unstructured::new(fuzz_data);
    let mut config = u.arbitrary()?;
    let (generated, known_valid) = gen_module(&mut config, &mut u)?;
    let module = module_in_input.unwrap_or(&generated);
    if let Ok(file) = std::env::var("WRITE_FUZZ_INPUT_TO") {
        std::fs::write(file, encode_module(&module, &fuzz_data)).unwrap();
    }
    let known_valid = if module_in_input.is_some() {
        KnownValid::No
    } else {
        known_valid
    };
    run(module, known_valid, config, &mut u)
}

/// Used as part of `execute` above to determine whether a module is known to
/// be valid ahead of time.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum KnownValid {
    /// This module is known to be valid so it should assert compilation
    /// succeeds for example.
    Yes,
    /// This module is not known to be valid and it may not compile
    /// successfully. Note that it's also not known to compile unsuccessfully.
    No,
}

const SECTION_NAME: &str = "wasmtime-fuzz-input";

/// Implementation of a libfuzzer custom mutator for a single-module-fuzzer.
///
/// This mutator will take the seed specified in `data` and attempt to mutate
/// it with the provided `mutate` function. The `mutate` function may not
/// receive the `data` as-specified, but instead may receive only the seed
/// that was used to generate `data`.
pub fn mutate<T>(
    data: &mut [u8],
    mut size: usize,
    max_size: usize,
    gen_module: fn(&mut T, &mut Unstructured<'_>) -> Result<(Vec<u8>, KnownValid)>,
    mutate: fn(&mut [u8], usize, usize) -> usize,
) -> usize
where
    T: for<'a> Arbitrary<'a>,
{
    // If `data` is a valid wasm module with the fuzz seed at the end, then
    // discard the wasm module portion and instead shuffle the seed into the
    // beginning of the `data` slice. This is the "de-envelope" part of the
    // seed management here.
    //
    // After this the `data` array should contain the raw contents used to
    // produce the module and is ripe for mutation/minimization/etc.
    if let Ok(input) = extract_fuzz_input(&data[..size]) {
        let start = input.fuzz_data.as_ptr() as usize - data.as_ptr() as usize;
        size = input.fuzz_data.len();
        data.copy_within(start..start + input.fuzz_data.len(), 0);
    }

    // Delegate to the provided mutation function for standard mutations to
    // apply.
    let new_size = mutate(data, size, max_size);

    // Next the goal of this function is to produce a test case which is an
    // actual wasm module. To that end this will run module generation over the
    // input provided. If this is all successful then the custom section
    // representing the seed is appended to the module, making it a sort of
    // self-referential module.
    //
    // After all this it's copied into `data` if the it fits. If the module
    // doesn't fit then the seed is left un-perturbed since there's not much
    // that we can do about that.
    let mut u = Unstructured::new(&data[..new_size]);
    match u
        .arbitrary()
        .and_then(|mut config| gen_module(&mut config, &mut u))
    {
        Ok((module, _known_valid)) => {
            let module = encode_module(&module, &data[..new_size]);

            if module.len() < max_size {
                log::debug!(
                    "successfully generated mutated module with \
                     appended input section"
                );
                data[..module.len()].copy_from_slice(&module);
                return module.len();
            } else {
                log::debug!("mutated module doesn't fit in original slice");
            }
        }

        // If our new seed can't generate a new module then that's something
        // for the fuzzer to figure out later when it "officially" executes
        // this fuzz input. For the purposes of this function it's not too
        // useful to try to put it in an envelope otherwise so ignore it.
        Err(e) => {
            log::debug!("failed to generate module from mutated seed {e:?}");
        }
    }

    new_size
}

fn encode_module(module: &[u8], fuzz_data: &[u8]) -> Vec<u8> {
    let mut module = module.to_vec();
    wasm_encoder::CustomSection {
        name: SECTION_NAME.into(),
        data: Cow::Borrowed(&fuzz_data),
    }
    .append_to(&mut module);
    module
}

struct FuzzInput<'a> {
    /// The module extracted from the input, without the fuzz input custom
    /// section.
    module: &'a [u8],

    /// The contents of the fuzz input custom section.
    fuzz_data: &'a [u8],
}

/// Attempts to extract a fuzz input from the `data` provided.
///
/// This will attempt to read `data` as a WebAssembly binary. If successful
/// and the module ends with a custom section indicating it's a fuzz input
/// then the contents of the custom section are returned along with the
/// contents of the original module.
fn extract_fuzz_input(data: &[u8]) -> anyhow::Result<FuzzInput<'_>> {
    use wasmparser::{Parser, Payload};
    let mut prev_end = 8;
    for section in Parser::new(0).parse_all(data) {
        let section = section?;

        // If this is a custom section, the end of the section is the end of
        // the entire module, and it's got the expected name, then this section
        // is assumed to be the input seed to the fuzzer.
        //
        // The section's contents are returned through `fuzz_data` and the wasm
        // binary format means that we can simply chop off the last custom
        // section and still have a valid module.
        if let Payload::CustomSection(s) = &section {
            if s.name() == SECTION_NAME && s.range().end == data.len() {
                return Ok(FuzzInput {
                    module: &data[..prev_end],
                    fuzz_data: s.data(),
                });
            }
        }

        // Record each section's end to record what the end of the module is
        // up to this point.
        if let Some((_, range)) = section.as_section() {
            prev_end = range.end;
        }
    }
    anyhow::bail!("no input found")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::{RngCore, SeedableRng};

    #[test]
    fn changing_configuration_does_not_change_module() {
        drop(env_logger::try_init());

        // This test asserts that if the static configuration associated with a
        // module changes then the generated module, as sourced from the
        // original fuzz input, does not change. That's the whole purpose of
        // this module, to enable our fuzz inputs to be in a format that's
        // resilient to changes in configuration over time (or at least the
        // module part of the input).
        //
        // This test will execute N=200 iterations where each iteration will
        // attempt to, with some fresh random data, generate a module. This
        // module is then "mutated" with a noop mutation to effectively
        // serialize it into the envelope where the module is preserved. The
        // now-mutated input, which should be a wasm module, is then passed
        // as the seed to a second execution which has a different static input.
        //
        // This simulates having a fuzzer one day produce an interesting test
        // case through mutation, and then the next day the configuration of
        // the fuzzer changes. On both days the module input to the function
        // should have been the same.

        let mut rng = SmallRng::seed_from_u64(0);
        let max_size = 4096;
        let seed_size = 128;
        let mut buf = vec![0; max_size];
        let mut compares = 0;
        for _ in 0..200 {
            rng.fill_bytes(&mut buf[..seed_size]);

            let run1 = run_config::<u32>;
            let mutate = mutate::<u32>;
            let run2 = run_config::<(u32, u32)>;

            if let Ok((module, known_valid)) = execute(&buf[..seed_size], run1, generate) {
                assert_eq!(known_valid, KnownValid::Yes);
                let new_size = mutate(&mut buf, seed_size, max_size, generate, noop_mutate);
                if let Ok((module2, known_valid)) = execute(&buf[..new_size], run2, generate) {
                    assert_eq!(known_valid, KnownValid::No);
                    compares += 1;
                    if module != module2 {
                        panic!("modules differ");
                    }
                }
            }
        }

        // At least one iteration should have succeeded in the fuzz generation
        // above.
        assert!(compares > 0);

        fn run_config<T>(
            data: &[u8],
            known_valid: KnownValid,
            _: T,
            _: &mut Unstructured<'_>,
        ) -> Result<(Vec<u8>, KnownValid)>
        where
            T: for<'a> Arbitrary<'a>,
        {
            Ok((data.to_vec(), known_valid))
        }

        fn generate<T>(_: &mut T, u: &mut Unstructured<'_>) -> Result<(Vec<u8>, KnownValid)>
        where
            T: for<'a> Arbitrary<'a>,
        {
            Ok((
                u.arbitrary::<wasm_smith::Module>()?.to_bytes(),
                KnownValid::Yes,
            ))
        }

        fn noop_mutate(_buf: &mut [u8], size: usize, _new_size: usize) -> usize {
            size
        }
    }
}
