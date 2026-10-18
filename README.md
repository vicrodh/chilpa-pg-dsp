# Chilpa DSP

Preset labels are available in English and Spanish. The host chooses the
display text from the interface language, including preset names, categories,
descriptions and source notes. Language changes leave the processing parameters
and selected preset IDs intact. The catalog's optional `localizations` map
is keyed by language and then preset ID. Each entry can override `name`,
`category`, `description` and `source`; filters and all other DSP parameters
remain in the canonical preset.

This sandboxed WASM plugin provides 111 offline listening presets. The
Chilpa host evaluates validated parameters in its PCM renderer. Guest code
never runs on the audio thread and never receives music files, accounts or PCM.

**DSP changes the signal and breaks strict bit-perfect playback.** This
notice appears in the plugin description and editor. The quality flyout names the
preset while processing is active and reports processed output.

Requires Chilpa 0.1.0-beta or later with DSP support, graphic equalizer and
measured filters with Q from 0.1 to 12 and gain from -24 to +18 dB. Older hosts can reject this catalog. This package does not add
processing to an older host by itself.

## Presets and controls

Preset categories are Base, Tone, Headphones, Speakers, Music, Daily use,
Channels and Soundbars. The editor has
search, persistent favorites, up to 64 personal presets, and separate saved
selections for each output device. The player shows a DSP control beside Cast
only while an approved DSP plugin is available.

The 58 measured AutoEq profiles cover HD650, Edition XS, PXC 550 and
PXC 550-II, plus headphones and earphones from Sony, Bose, Apple, Sennheiser, Audio-Technica, Beyerdynamic, AKG, HIFIMAN,
Focal, Meze, Audeze, Philips, Shure, Samsung, Anker, Google, Jabra, Technics,
Moondrop, Truthear, 7Hz and Etymotic. Exact model names, measurement providers
and source paths are listed in `profiles/catalog.json`. The published filter parameters and preamp are preserved. The host can reduce
preamp further to reserve one dB above the combined EQ response. The PXC
550 profile needs its stated wireless/ANC mode; different pads, fit, device
effects and individual units change the result.

The generic Soundbar 2.1 preset targets clearer voices and less bass buildup.
It is a starting point for Creative Stage V2, without measured correction for
that model or room calibration.

The default editor provides ten vertical sliders from 31 Hz to 16 kHz,
with a ±12 dB range, a preamp, reset and bypass. A measured parametric profile
keeps its precise filters until an explicit switch to graphic EQ. The previous
profile and draft remain available for restoration, including after a restart. Advanced controls and import/export remain available.

Processing supports up to ten parametric filters (peak, low/high shelf, low/high pass,
notch), filtered crossfeed, stereo narrowing/mono, balance, channel polarity,
up to 30 ms channel delay and linked peak compression without makeup gain.
The stereo matrix applies to two-channel PCM; EQ and compression apply to
all supported channels. A 10 ms crossfade smooths preset and bypass changes.
Filter histories persist between contiguous tracks, including audio CDs,
and reset after seeking or skipping. Processing follows read-ahead and
precedes device writes, so partial writes never apply a filter twice.

Bypass and the flat preset preserve the original packed samples. Native DSD
and DoP bypass DSP; DSD already converted to PCM can be processed. Cast sends
the original audio and displays that local DSP does not apply. The chosen DSD mode and output sample rate remain unchanged. A band at or above 49% of
the playback rate is skipped instead of being shifted into another range.

The renderer reserves headroom from the combined filter response and clamps
unexpected out-of-range output samples, counting them in the editor. This is a sample
guard, not a true-peak limiter or a guarantee against intersample peaks.
Convolution, automatic room measurement, VST/AU hosting and dynamic loudness
compensation are outside this version.

## Build and package

The vendored SDK is pinned to
`f3331ad97593c23277351e4cc1c752f9b2719028` (`protocol-4`), matching the
host's vendored API. Source provenance is in `vendor/ORIGIN.json`. Only the
Cargo manifests are adapted for standalone use. The plugin declares API version 1 and
DSP catalog protocol 1; these numbers are separate from the SDK tag.

With Rust 1.90+ and the `wasm32-unknown-unknown` target installed:

```sh
cargo test --offline -p chilpa-pg-dsp --lib
tools/build-plugin
```

The build produces `dist/app.chilpa.dsp/` and `dist/chilpa-dsp-0.1.1.zip`,
including licenses and profile provenance.

Install the ZIP through the updated host's plugin manager and approve its
single permission: `process_audio`. It requests no network, filesystem,
library, account or cross-plugin permission. The host owns preferences.
The first selection is flat/bypass; choosing a tonal preset enables DSP.

The editor imports the parametric text subset used by AutoEq and common EQ
exporters (`Preamp`, `PK`, `LS`/`LSC`, `HS`/`HSC`, `HP`/`HPQ`, `LP`/`LPQ`,
`NO`) or an exported Chilpa preset JSON. Unknown active filter types and
unsupported directives reject the entire import. JSON export uses the clipboard, without writing arbitrary files.

Run the packaged host smoke in an isolated profile:

```sh
CHILPA_DSP_PACKAGE=/absolute/path/to/dist/app.chilpa.dsp \
  cargo test -q -p chilpa-api --test dsp
```

That test loads the exact WASM, checks approval and catalog validation,
selects all five equipment presets, checks every factory preset at six
sample rates, persists favorites and personal presets, restarts the host,
and revokes the plugin. DSP renderer tests also check real PCM output,
partial writes, live bypass, native DSD and DoP preservation.

## References and licenses

Feature references include [FusionDSP](https://help.volumio.com/en/articles/99-how-to-use-fusiondsp-plugin),
[Audirvana's processing suite](https://audirvana.com/signal-processing-suite/)
and [SqueezeDSP](https://github.com/Foxenfurter/SqueezeDSP). The
[Navidrome plugin isolation and permission model](https://www.navidrome.org/docs/usage/features/plugins/)
served as an integration reference; that documented capability list does
not expose a comparable DSP contract. No DSP code from these projects is included.

The filters follow the [RBJ Audio EQ Cookbook](https://www.w3.org/TR/audio-eq-cookbook/).
The 53 generic presets are original listening adjustments, without claims
of measured correction. The 58 measured profiles come from
[AutoEq](https://github.com/jaakkopasanen/AutoEq) at immutable revision
`7ae0f56d53074872b028649617a22bbb4232feb7`. Exact source paths, measurement
providers and URLs are in `profiles/ORIGIN.json`; the input filter files
are preserved in `profiles/autoeq/`. AutoEq's MIT license is included in
`licenses/AutoEq-MIT.txt` and in the distributable. Regenerate the catalog
with `tools/make-presets` without network access. To refresh the preserved
input files, `tools/fetch-profiles` uses `gh` and requires `GH_CONFIG_DIR` to
select the authenticated GitHub configuration; the revision remains pinned.

MIT; see `LICENSE`.
