# skrifa

[![Crates.io](https://img.shields.io/crates/v/skrifa.svg)](https://crates.io/crates/skrifa)
[![Docs](https://docs.rs/skrifa/badge.svg)](https://docs.rs/skrifa)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)

Skrifa ships as the font rendering backend for Chrome on Android and Linux,
and on other platforms for some font formats. See [Memory safety for web
fonts](https://developer.chrome.com/blog/memory-safety-fonts).

Skrifa draws glyphs from fonts at a size, optionally hinted, and reads
additional metadata: character mapping, metrics, names and variations. It is
built on the
[read-fonts](https://github.com/googlefonts/fontations/tree/main/read-fonts)
parser, is part of the [oxidize](https://github.com/googlefonts/oxidize)
project, and is tested glyph by glyph against FreeType.

```rust
use skrifa::{
    font::{glyph::outline::SvgSink, Font},
    DrawMemory, HintOptions, RasterTarget, ScalerFont,
};

fn draw_a(data: Vec<u8>) -> Result<(), skrifa::ScaleError> {
    // Load the first font in the file and create a bold instance.
    let font = Font::new(data, 0).unwrap();
    let bold = font.instance_builder().variations([("wght", 700.0)]).build();

    // Find the glyph for 'A'.
    let glyph_id = bold.charmap().map_unicode('A').unwrap_or_default();

    // Hint with the font's own instructions for a horizontal subpixel
    // target, at 16 pixels per em.
    let options = HintOptions::interpreter().with_target(RasterTarget::Lcd);
    let scaler = ScalerFont::hinted(bold, 16.0, options);

    if let Some(outline) = scaler.outline(glyph_id) {
        let mut path = SvgSink::new();
        let metrics = outline.draw(DrawMemory::Scratch, &mut path)?;
        println!("{path} advances by {:?}", metrics.h_advance);
    }
    Ok(())
}
```

## Fonts

A `Font` is a font at one location in its variation space. It loads from
owned or shared bytes, or from a function that supplies tables by tag, as a
platform font API does. It reads:

- OpenType and TrueType fonts and collections
- Standalone CFF font sets, including CID-keyed fonts
- Type 1 fonts, PFA or PFB, including multiple master fonts

The `font` module covers the rest of a font: the character map with Unicode
variation sequences, localized names, variation axes and named instances,
line, style and glyph metrics, glyph names, color palettes, color glyphs and
bitmap strikes.

## Glyphs

| Table    | Read  | Variations | Hinting                        |
|----------|-------|------------|--------------------------------|
| `glyf`   | ✔️    | ✔️          | TrueType interpreter, automatic |
| `CFF`    | ✔️    | -          | PostScript hints, automatic     |
| `CFF2`   | ✔️    | ✔️          | PostScript hints, automatic     |
| Type 1   | ✔️    | ✔️          | automatic                       |
| `VARC`   | ✔️    | ✔️          | automatic                       |
| `COLR`   | ✔️    | ✔️ (v1)     | -                               |
| `EBDT`   | ✔️    | -          | -                               |
| `CBDT`   | ✔️    | -          | -                               |
| `sbix`   | ✔️    | -          | -                               |

Outlines draw through `ScalerFont` into any `PathSink`. `COLR` glyphs paint
through a `PaintSink`. Bitmap glyphs come as coverage masks, BGRA pixels or
PNG data for the caller to decode.

## Hinting

`HintOptions` chooses between three engines:

- **The TrueType interpreter** runs a font's instructions, with the classic
  ClearType rules or the newer ones. The newer rules also follow the font's
  `gasp` table and let glyphs that ask for it keep fractional horizontal
  positions.
- **The PostScript hinter** runs the hints in CFF and CFF2 charstrings.
- **The automatic hinter** derives hints from the outline, for any outline
  format.

Each hints for a target: monochrome, grayscale, light grayscale, or
horizontal or vertical subpixel. A TrueType font that needs its
instructions to draw correct contours always runs them.

## Fork compatibility switches

This fork provides process-local compatibility experiments through the
comma-separated `SKRIFA_GDI_RESEARCH` environment variable. Set it before
the first interpreter call; the selection is cached for the process.
Experiments are disabled when unselected, and in `no_std` builds.

`getinfo-gdi42` selects the measured GDI monochrome `GETINFO` profile:
version 42, font-variation support (selector `0x8`, result `0x400`), and
vertical phantom-point support (selector `0x10`, result `0x800`). Support
is reported for both static and variable fonts. Existing rotation and
stretching answers still apply. Smooth targets keep their existing
answers, and `GETVARIATION` execution is unchanged. This profile does not
claim complete GDI rendering equivalence.

`scanctrl-gdi32` selects the measured monochrome dropout policy: without a
font override, simple dropout including stubs is active through 32 ppem and
inactive above that size. Each explicit `SCANCTRL` word is evaluated from an
inactive condition state, so an unmet enabling condition does not retain an
earlier activation. Fonts may still enable or disable dropout explicitly;
`SCANTYPE` continues to select the algorithm. Smooth targets and unselected
builds retain their existing behavior.

The adoption check covered 16,331 natural, 2,579 synthetic and 9,885 variable
cases against unchanged GDI references. Eight synthetic cases become newly
pixel-exact, leaving no differing pixels in the natural or synthetic corpus;
the variable corpus retains its 516 differing pixels. No case regresses and
no outline or advance changes. Independent native checks also cover 180
state combinations and 1,071 fixed-outline cases across 153 sizes. Private
fonts, captures, decompilation and replay artifacts remain outside this repo.

`gvar-inferred-truncate` selects GDI-compatible inference for unreferenced
points in sparse `gvar` tuples. The `read-fonts` interpolation computes each
inferred delta directly from the reference deltas, then truncates it toward
zero in design units before adding the original coordinate. This avoids
quantizing a position slope before truncation. Explicit deltas, clamping
outside the reference interval, and single-reference contour shifts retain
their existing behavior. The switch uses the same process-local environment
selection and remains disabled in unselected and `no_std` builds. Set the
selection before the first outline or variation-interpolation call.

Measured independently of the raster correction, this profile reduces the
9,885-case variable corpus from 212 to 29 differing pixels: 134 cases improve,
132 become newly pixel-exact, and none regress. Contours improve in 718 cases,
including 708 newly exact, with no lost exact contours. Advances are unchanged.
The test fonts and GDI captures are local evidence and are not redistributed.
All 16,331 natural and 2,579 synthetic cases remain pixel-exact; their native
contour references also stay exact. Disabling the profile reproduces the
previous fork's complete glyph records across all three corpora.

`cvt-full-precision` preserves all scale-factor bits when scaling the control
value table. A signed 64-bit product combines the 26.6 CVT value with the full
16.16 scale, followed by one 22-bit shift. Exact halves round toward positive
infinity. This profile includes that rounding rule independently of `cvt-ties`;
it applies to both static CVTs and CVTs adjusted by `cvar`. It is disabled when
unselected and in `no_std` builds.

The measured case is a control value of -471 design units at 100 ppem with
2250 units/em: the reduced-scale path gives -1339 in 26.6 pixels, while the
full-scale path and GDI give -1340. No font-name or glyph-ID exception is used.

With the renderer at `18c8b25` and its preceding eighteen selections, enabling
this profile reduces the 9,885-case variable corpus from 23 to 16 differing
pixels. Four cases become newly pixel-exact, with no regressions or lost exact
cases. Contours improve in 21 cases, including ten newly exact; no contour
regresses or loses exactness. Advances are unchanged. The clean fork's complete
glyph records match the successful local research prototype.
All 16,331 natural and 2,579 synthetic cases remain pixel-exact, with unchanged
native contour references. Disabling the new profile reproduces the preceding
fork's complete glyph records across all three corpora.

All 240 independent geometric controls match GDI in both contours and masks.
Three unit tests cover low scale bits, fractional design units, signed half
ties and agreement with reduced scaling when the low scale bits are zero.
The library passes 519 unit tests and its `no_std` check. Private fonts, corpus
manifests, raw GDI captures and research instrumentation remain outside Git.

## Features

- `std` (default): use the standard library. Without it, enable `libm`
  for `no_std`.
- `agl`: map characters to the glyphs of Type 1 and standalone CFF fonts
  through their glyph names, with the Adobe Glyph List. Without it those
  fonts map no characters.
- `type1`: enable Adobe Type 1 font support. Also enables `agl`.

## Panicking

Skrifa should not panic, whatever the font data or the way it is called.
Please file an issue if it does.

## Safety

Unsafe code is forbidden by a `#![forbid(unsafe_code)]` attribute in the root
of the library.

## The name?

Following along with our theme, *skrifa* is Old Norse for "write" or "it is
written." And so it is named.

## License

Licensed under either of the [Apache License, Version
2.0](https://www.apache.org/licenses/LICENSE-2.0) or the [MIT
license](https://opensource.org/licenses/MIT), at your option.
