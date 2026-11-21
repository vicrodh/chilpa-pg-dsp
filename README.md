# Chilpa DSP

This update targets **Chilpa 0.1.3** and its matching plugin contracts.
Release and catalog publication are pending that host release.

Chilpa DSP 0.2.0 supplies **8,825 offline presets**: 8,772 measured AutoEq
profiles from 5,981 distinct model names, plus 53 listening presets. Multiple
measurement sources and headphone variants remain separate entries.

**The full 0.2 editor and processing controls require the matching updated
Chilpa host with DSP bootstrap protocol 2.** Installing the ZIP alone cannot
update a host's editor or PCM renderer. Older DSP hosts can continue loading
the 111-preset compatibility catalog through protocol 1.

The plugin is a data-only WASM package. The host validates parameters, owns
preferences and processes local PCM. Guest code never receives PCM, music
files, library data or accounts. The package requests only `process_audio`,
with no network or storage permissions. The same ZIP is used by native and
Flatpak hosts and by supported operating systems.

**DSP changes the signal and breaks strict bit-perfect playback.** Bypass and
neutral processing preserve packed PCM. Native DSD and DoP bypass DSP; DSD
already converted to PCM can be processed. Cast sends the original audio.

## Controls in the updated host

The larger modal has internal sidebar navigation and separate scrolling
sections. Narrow windows use compact navigation. Only the current editor is
mounted, and closing the modal releases it.

- Independent Bass and Treble controls, each with gain, frequency and Q.
  Bass supports -24 to +22.5 dB and 20–250 Hz; Treble supports -24 to +15 dB
  and 5–15 kHz. Q is 0.1–2. Zero dB is neutral. Tone does not replace the
  headphone correction filters.
- Up to 64 parametric filters, separate from the two tone shelves: peak,
  low/high shelf, low/high pass, notch and band pass. Each filter has enable,
  channel, lock and color-group controls. Filters can be reordered. Mouse
  dragging adjusts frequency/gain and scrolling over a point adjusts Q.
- Graphic EQ with 5–32 logarithmically spaced bands, configurable frequency
  endpoints and a ±12 dB range. Rebuilding starts flat and retains the
  previous correction for restoration, including after restart.
- Independent EQ, Tone and global bypass. Requested preamp spans -60 to
  +24 dB. Automatic headroom defaults on, includes tone and the actual
  channel routing, and can reduce the requested gain. It does not cancel
  intentional dynamics pre-gain or makeup gain.
- Crossfeed, stereo narrowing/mono, balance, polarity and up to 30 ms delay
  per stereo channel. The stereo matrix applies only to two-channel PCM;
  EQ and linked dynamics support up to 32 channels and 1536 kHz PCM.
- Linked peak compressor with threshold, ratio, attack, release, knee,
  pre-gain, makeup, detector high-pass, downward expander and gate.
- Separate zero-latency sample-peak limiter with ceiling and release. It
  limits after configuration crossfades and retains the last input sample.
  It is not a true-peak limiter. The final sample guard remains a separate
  safety boundary.
- Measured peak-hold input/output levels, compression/limiter reduction and
  a resettable clipped-sample counter.

Live gestures are grouped into at most one preview every 50 ms and commit
on release. Banks are prepared outside delivery; changes preserve compatible
histories. Presentation-only changes preserve the running bank. Preset and
bypass changes retain a 10 ms transition.

## Presets, assignments and interchange

Search operates on a compact offline index and returns bounded pages rather
than loading thousands of profiles into the host. A query can match a model,
measurement provider or variant. The existing measured IDs remain stable.
The catalog uses the immutable AutoEq revision recorded in
`profiles/library/ORIGIN.json`; unsupported profiles are listed there with a
reason. Headphone fit, pads, seal, device mode and individual units affect
measured corrections. Generic speaker presets are starting points, not room
calibration or measured correction for a particular speaker.

The host provides favorites and up to 64 personal presets, with save-copy,
rename, replace and batch deletion. Optional autosave updates a personal
preset after adjustment; factory presets retain a separate per-output draft.
Exports preserve tone, channels and processing settings. Device assignments
take priority over explicitly assigned device classes, then the general
default. Chilpa does not guess headphones attached to a DAC.

The editor accepts AutoEq parametric text, a Chilpa preset, arrays and
versioned Chilpa preset bundles. Batch imports are validated as a whole.
A bounded Poweramp parametric JSON adapter accepts the two exported tone
bands and peak/shelf EQ bands, preserving channels and translating the
exporter's normalized gain semantics. Graphic/envelope exports and external
pass-filter gain semantics are rejected rather than approximated. Import
compatibility does not claim acoustic equivalence between engines.

## Build

The vendored ABI/SDK is pinned to
`f3331ad97593c23277351e4cc1c752f9b2719028` (`protocol-4`), matching the
host's vendored API. `vendor/ORIGIN.json` records the source. Manifest API
version 1, bootstrap/search protocol 2 and SDK Git tag are separate values.
No SDK sources were changed.

With Rust 1.90+ and the `wasm32-unknown-unknown` target:

```sh
cargo test --offline -p chilpa-pg-dsp --lib
tools/build-plugin
```

The result is `dist/chilpa-dsp-0.2.0.zip`, with its WASM, manifest, licenses
and both compatibility and expanded-catalog provenance. Install it through
the matching host's plugin manager and approve `process_audio`.

To regenerate the measured catalog, select an authenticated `gh` profile,
run `tools/fetch-catalog --cache /absolute/cache/path`, then
`tools/pack-catalog`. Retrieval uses immutable Git objects and verifies their
blob digests. `profiles/library/presets.jsonl` is the canonical input;
`index.tsv` and `catalog.bin` are reproducible generated files. A guest
unpacks at most one 64-profile shard at a time.

## Licenses

The plugin and the AutoEq data are MIT licensed. The package includes the
AutoEq license and source revision/digests. The offline compression dependency
uses the Rust flate2 backend (MIT/Apache-2.0). Poweramp's database, code,
artwork and presets are not included.

## Pull request packages

GitHub Actions runs tests and packages this branch on pull requests and pushes
to main. Download the ZIP, SHA-256 and archive-check report from the successful
run's artifacts. These are test artifacts retained for 30 days, not releases;
the workflow does not publish a release or update a plugin catalog.

CI checks do not replace installation of the exact ZIP in a compatible native
and Flatpak host. A build does not establish live account playback or physical
audio output. Release validation remains separate.

CI produces one platform-independent WASM ZIP and checks catalog regeneration.
