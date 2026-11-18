# Third-Party Licenses & Attributions

**Vibe FFmpeg** ([N213S/vibe-ffmpeg-ui](https://github.com/N213S/vibe-ffmpeg-ui)) is licensed under the [MIT License](LICENSE).

This document lists the direct and transitive Rust crates compiled into the Windows (`x86_64-pc-windows-msvc`) release build (`Cargo.lock`), bundled default UI fonts, and external runtime tools.

> **Apache-2.0 / Dual-License Election**: Where a dependency is dual-licensed (for example `MIT OR Apache-2.0`, `Apache-2.0 OR GPL-2.0-only` on `self_cell`, or `MIT OR Apache-2.0 OR LGPL-2.1-or-later` on `r-efi`), Vibe FFmpeg elects the permissive `MIT` or `Apache-2.0` terms.

---

## 1. Direct Rust Dependencies

| Crate | Version | Purpose | License | Author(s) / Upstream |
| --- | --- | --- | --- | --- |
| `eframe` | 0.37.0 | Cross-platform `egui` desktop app framework (`glow`, `system_fonts`, `default_fonts`) | `MIT OR Apache-2.0` | Emil Ernerfeldt ([Repository](https://github.com/emilk/egui/tree/main/crates/eframe)) |
| `egui` | 0.37.0 | Immediate-mode GUI library (re-exported via `eframe`) | `MIT OR Apache-2.0` | Emil Ernerfeldt ([Repository](https://github.com/emilk/egui)) |
| `rfd` | 0.18.0 | Native OS file and folder picker dialogs | `MIT` | Poly ([Repository](https://github.com/PolyMeilex/rfd)) |
| `ureq` | 3.5.2 | Minimal HTTPS client for official FFmpeg release checks (`native-tls`) | `MIT OR Apache-2.0` | Martin Algesten, Jacob Hoffman-Andrews ([Repository](https://github.com/algesten/ureq)) |
| `serde` | 1.0.229 | Serialization/deserialization framework (`derive`) | `MIT OR Apache-2.0` | Erick Tryzelaar, David Tolnay ([Repository](https://github.com/serde-rs/serde)) |
| `serde_json` | 1.0.154 | JSON parser for `ffprobe` metadata output | `MIT OR Apache-2.0` | Upstream contributors ([Repository](https://github.com/serde-rs/json)) |

---

## 2. External Runtime Tools (Not Bundled)

- **FFmpeg & ffprobe** ([ffmpeg.org](https://ffmpeg.org/) / Windows builds at [gyan.dev](https://www.gyan.dev/ffmpeg/builds/)) — External CLI executables invoked at runtime as separate OS child processes (`std::process::Command`). Vibe FFmpeg does **not** statically or dynamically link FFmpeg libraries (`libavcodec`, `libavformat`, etc.) and does **not** bundle FFmpeg binaries in its source repository. Depending on your local build configuration, FFmpeg binaries are typically licensed under **LGPL-2.1+** or **GPL-3.0+**.
- **Windows System Fonts** — On Windows, `src/ui/theme.rs` reads installed system fonts (`Segoe UI`, `Leelawadee UI`, `Tahoma`, `Arial`, `Malgun Gothic`, `Yu Gothic` / `Meiryo`, `Microsoft YaHei`, `Microsoft JhengHei`) locally from `C:\Windows\Fonts` at runtime for multi-script display and does **not** redistribute Microsoft font files.

---

## 3. Embedded Default Fonts (`epaint_default_fonts`)

Because `eframe` enables `"default_fonts"`, `epaint_default_fonts 0.37.0` (`(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`) embeds the following default font assets into compiled binaries:

- `Hack-Regular.ttf` — Hack (`MIT`) / DejaVu (`Public Domain`) / Bitstream Vera (`Bitstream Vera License`)
- `Ubuntu-Light.ttf` — Ubuntu Font Licence Version 1.0 (`Ubuntu-font-1.0`)
- `NotoEmoji-Regular.ttf` — SIL Open Font License Version 1.1 (`OFL-1.1`)
- `emoji-icon-font.ttf` / `egui-icons.ttf` — MIT License (Copyright (c) 2014 John Slegers)

<details>
<summary>Full Font & Special Crate License Notices</summary>

### SIL Open Font License 1.1 (`OFL.txt`)
```text
This Font Software is licensed under the SIL Open Font License,
Version 1.1.

This license is copied below, and is also available with a FAQ at:
http://scripts.sil.org/OFL

-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font
creation efforts of academic and linguistic communities, and to
provide a free and open framework in which fonts may be shared and
improved in partnership with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply to
any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software
components as distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to,
deleting, or substituting -- in part or in whole -- any of the
components of the Original Version, by changing formats or by porting
the Font Software to a new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed,
modify, redistribute, and sell modified and unmodified copies of the
Font Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components, in
Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the
corresponding Copyright Holder. This restriction only applies to the
primary font name as presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created using
the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
```

### Ubuntu Font Licence 1.0 (`UFL.txt`)
```text
-------------------------------
UBUNTU FONT LICENCE Version 1.0
-------------------------------

PREAMBLE
This licence allows the licensed fonts to be used, studied, modified and
redistributed freely. The fonts, including any derivative works, can be
bundled, embedded, and redistributed provided the terms of this licence
are met. The fonts and derivatives, however, cannot be released under
any other licence. The requirement for fonts to remain under this
licence does not require any document created using the fonts or their
derivatives to be published under this licence, as long as the primary
purpose of the document is not to be a vehicle for the distribution of
the fonts.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this licence and clearly marked as such. This may
include source files, build scripts and documentation.

"Original Version" refers to the collection of Font Software components
as received under this licence.

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to
a new environment.

"Copyright Holder(s)" refers to all individuals and companies who have a
copyright ownership of the Font Software.

"Substantially Changed" refers to Modified Versions which can be easily
identified as dissimilar to the Font Software by users of the Font
Software comparing the Original Version with the Modified Version.

To "Propagate" a work means to do anything with it that, without
permission, would make you directly or secondarily liable for
infringement under applicable copyright law, except executing it on a
computer or modifying a private copy. Propagation includes copying,
distribution (with or without modification and with or without charging
a redistribution fee), making available to the public, and in some
countries other activities as well.

PERMISSION & CONDITIONS
This licence does not grant any rights under trademark law and all such
rights are reserved.

Permission is hereby granted, free of charge, to any person obtaining a
copy of the Font Software, to propagate the Font Software, subject to
the below conditions:

1) Each copy of the Font Software must contain the above copyright
notice and this licence. These can be included either as stand-alone
text files, human-readable headers or in the appropriate machine-
readable metadata fields within text or binary files as long as those
fields can be easily viewed by the user.

2) The font name complies with the following:
(a) The Original Version must retain its name, unmodified.
(b) Modified Versions which are Substantially Changed must be renamed to
avoid use of the name of the Original Version or similar names entirely.
(c) Modified Versions which are not Substantially Changed must be
renamed to both (i) retain the name of the Original Version and (ii) add
additional naming elements to distinguish the Modified Version from the
Original Version. The name of such Modified Versions must be the name of
the Original Version, with "derivative X" where X represents the name of
the new work, appended to that name.

3) The name(s) of the Copyright Holder(s) and any contributor to the
Font Software shall not be used to promote, endorse or advertise any
Modified Version, except (i) as required by this licence, (ii) to
acknowledge the contribution(s) of the Copyright Holder(s) or (iii) with
their explicit written permission.

4) The Font Software, modified or unmodified, in part or in whole, must
be distributed entirely under this licence, and must not be distributed
under any other licence. The requirement for fonts to remain under this
licence does not affect any document created using the Font Software,
except any version of the Font Software extracted from a document
created using the Font Software may only be distributed under this
licence.

TERMINATION
This licence becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM OTHER
DEALINGS IN THE FONT SOFTWARE.
```

### Hack / Bitstream Vera (`Hack-Regular.txt`)
```text
The work in the Hack project is Copyright 2018 Source Foundry Authors and licensed under the MIT License

The work in the DejaVu project was committed to the public domain.

Bitstream Vera Sans Mono Copyright 2003 Bitstream Inc. and licensed under the Bitstream Vera License with Reserved Font Names "Bitstream" and "Vera"
MIT License

Copyright (c) 2018 Source Foundry Authors

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
BITSTREAM VERA LICENSE

Copyright (c) 2003 by Bitstream, Inc. All Rights Reserved. Bitstream Vera is a trademark of Bitstream, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy of the fonts accompanying this license ("Fonts") and associated documentation files (the "Font Software"), to reproduce and distribute the Font Software, including without limitation the rights to use, copy, merge, publish, distribute, and/or sell copies of the Font Software, and to permit persons to whom the Font Software is furnished to do so, subject to the following conditions:

The above copyright and trademark notices and this permission notice shall be included in all copies of one or more of the Font Software typefaces.

The Font Software may be modified, altered, or added to, and in particular the designs of glyphs or characters in the Fonts may be modified and additional glyphs or characters may be added to the Fonts, only if the fonts are renamed to names not containing either the words "Bitstream" or the word "Vera".

This License becomes null and void to the extent applicable to Fonts or Font Software that has been modified and is distributed under the "Bitstream Vera" names.

The Font Software may be sold as part of a larger software package but no copy of one or more of the Font Software typefaces may be sold by itself.

THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL BITSTREAM OR THE GNOME FOUNDATION BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM OTHER DEALINGS IN THE FONT SOFTWARE.

Except as contained in this notice, the names of Gnome, the Gnome Foundation, and Bitstream Inc., shall not be used in advertising or otherwise to promote the sale, use or other dealings in this Font Software without prior written authorization from the Gnome Foundation or Bitstream Inc., respectively. For further information, contact: fonts at gnome dot org.
```

### `emoji-icon-font` / `egui-icons` (`emoji-icon-font-mit-license.txt`)
```text
MIT License

Copyright (c) 2014 John Slegers

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```

### `libloading` (`ISC`)
```text
Copyright © 2015, Simonas Kazlauskas

Permission to use, copy, modify, and/or distribute this software for any purpose with or without
fee is hereby granted, provided that the above copyright notice and this permission notice appear
in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS
SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE
AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF
THIS SOFTWARE.
```

### `cfg_aliases` Notice (`NOTICES.md`)
```text
# 3rd Party Notices

The `cfg_aliases!` macro uses a lot of the code from [`tectonic_cfg_support::target_cfg!`] macro which is under the following license:

[`tectonic_cfg_support::target_cfg!`]: https://github.com/tectonic-typesetting/tectonic/blob/f2439b936470ad27bdf92882064bc4702ee01899/cfg_support/src/lib.rs#L166

    tectonic_cfg_support is licensed under the MIT License.

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the “Software”), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.
---
```

</details>

---

## 4. Complete Resolved Windows Crate List (184 Crates)

Generated from `cargo metadata --filter-platform x86_64-pc-windows-msvc`:

| Crate | Version | License | Upstream |
| --- | --- | --- | --- |
| `accesskit` | 0.25.2 | `MIT OR Apache-2.0` | https://github.com/AccessKit/accesskit |
| `accesskit_consumer` | 0.39.2 | `MIT OR Apache-2.0` | https://github.com/AccessKit/accesskit |
| `accesskit_windows` | 0.35.2 | `MIT OR Apache-2.0` | https://github.com/AccessKit/accesskit |
| `accesskit_winit` | 0.34.2 | `Apache-2.0` | https://github.com/AccessKit/accesskit |
| `adler2` | 2.0.1 | `0BSD OR MIT OR Apache-2.0` | https://github.com/oyvindln/adler2 |
| `ahash` | 0.8.12 | `MIT OR Apache-2.0` | https://github.com/tkaitchuck/ahash |
| `arboard` | 3.7.0 | `MIT OR Apache-2.0` | https://github.com/1Password/arboard |
| `arrayvec` | 0.7.8 | `MIT OR Apache-2.0` | https://github.com/bluss/arrayvec |
| `autocfg` | 1.5.1 | `Apache-2.0 OR MIT` | https://github.com/cuviper/autocfg |
| `base64` | 0.23.1 | `MIT OR Apache-2.0` | https://github.com/marshallpierce/rust-base64 |
| `base64ct` | 1.8.3 | `Apache-2.0 OR MIT` | https://github.com/RustCrypto/formats |
| `bitflags` | 2.13.2 | `MIT OR Apache-2.0` | https://github.com/bitflags/bitflags |
| `bytemuck` | 1.25.2 | `Zlib OR Apache-2.0 OR MIT` | https://github.com/Lokathor/bytemuck |
| `bytemuck_derive` | 1.12.1 | `Zlib OR Apache-2.0 OR MIT` | https://github.com/Lokathor/bytemuck |
| `byteorder-lite` | 0.1.0 | `Unlicense OR MIT` | https://github.com/image-rs/byteorder-lite |
| `bytes` | 1.12.1 | `MIT` | https://github.com/tokio-rs/bytes |
| `cfg_aliases` | 0.2.2 | `MIT` | https://github.com/katharostech/cfg_aliases |
| `cfg-if` | 1.0.5 | `MIT OR Apache-2.0` | https://github.com/rust-lang/cfg-if |
| `clipboard-win` | 5.4.3 | `BSL-1.0` | https://github.com/DoumanAsh/clipboard-win |
| `color` | 0.3.3 | `Apache-2.0 OR MIT` | https://github.com/linebender/color |
| `crc32fast` | 1.5.5 | `MIT OR Apache-2.0` | https://github.com/srijs/rust-crc32fast |
| `cursor-icon` | 1.2.0 | `MIT OR Apache-2.0 OR Zlib` | https://github.com/rust-windowing/cursor-icon |
| `der` | 0.8.3 | `Apache-2.0 OR MIT` | https://github.com/RustCrypto/formats |
| `displaydoc` | 0.2.7 | `MIT OR Apache-2.0` | https://github.com/yaahc/displaydoc |
| `document-features` | 0.2.12 | `MIT OR Apache-2.0` | https://github.com/slint-ui/document-features |
| `dpi` | 0.1.2 | `Apache-2.0 AND MIT` | https://github.com/rust-windowing/winit |
| `ecolor` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `eframe` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui/tree/main/crates/eframe |
| `egui` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `egui_glow` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui/tree/main/crates/egui_glow |
| `egui_system_font_provider` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui/tree/main/crates/egui_system_font_provider |
| `egui-winit` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui/tree/main/crates/egui-winit |
| `either` | 1.19.0 | `MIT OR Apache-2.0` | https://github.com/rayon-rs/either |
| `emath` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui/tree/main/crates/emath |
| `epaint` | 0.37.0 | `MIT OR Apache-2.0` | https://github.com/emilk/egui/tree/main/crates/epaint |
| `epaint_default_fonts` | 0.37.0 | `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0` | https://github.com/emilk/egui/tree/main/crates/epaint_default_fonts |
| `error-code` | 3.4.0 | `BSL-1.0` | https://github.com/DoumanAsh/error-code |
| `euclid` | 0.22.14 | `MIT OR Apache-2.0` | https://github.com/servo/euclid |
| `fax` | 0.2.7 | `MIT` | https://github.com/pdf-rs/fax |
| `fdeflate` | 0.3.7 | `MIT OR Apache-2.0` | https://github.com/image-rs/fdeflate |
| `fearless_simd` | 0.4.1 | `Apache-2.0 OR MIT` | https://github.com/linebender/fearless_simd |
| `flate2` | 1.1.11 | `MIT OR Apache-2.0` | https://github.com/rust-lang/flate2-rs |
| `foldhash` | 0.2.0 | `Zlib` | https://github.com/orlp/foldhash |
| `fontique` | 0.11.1 | `Apache-2.0 OR MIT` | https://github.com/linebender/parley |
| `font-types` | 0.12.7 | `MIT OR Apache-2.0` | https://github.com/googlefonts/fontations |
| `form_urlencoded` | 1.2.2 | `MIT OR Apache-2.0` | https://github.com/servo/rust-url |
| `getrandom` | 0.3.4 | `MIT OR Apache-2.0` | https://github.com/rust-random/getrandom |
| `gl_generator` | 0.14.0 | `Apache-2.0` | https://github.com/brendanzab/gl-rs/ |
| `glifo` | 0.3.0 | `Apache-2.0 OR MIT` | https://github.com/linebender/vello |
| `glow` | 0.17.0 | `MIT OR Apache-2.0 OR Zlib` | https://github.com/grovesNL/glow |
| `glutin` | 0.32.3 | `Apache-2.0` | https://github.com/rust-windowing/glutin |
| `glutin_egl_sys` | 0.7.1 | `Apache-2.0` | https://github.com/rust-windowing/glutin |
| `glutin_wgl_sys` | 0.6.1 | `Apache-2.0` | https://github.com/rust-windowing/glutin |
| `glutin-winit` | 0.5.0 | `MIT` | https://github.com/rust-windowing/glutin |
| `guillotiere` | 0.7.0 | `MIT/Apache-2.0` | https://github.com/nical/guillotiere |
| `half` | 2.7.1 | `MIT OR Apache-2.0` | https://github.com/VoidStarKat/half-rs |
| `harfrust` | 0.12.0 | `MIT` | https://github.com/harfbuzz/harfrust |
| `hashbrown` | 0.17.1 | `MIT OR Apache-2.0` | https://github.com/rust-lang/hashbrown |
| `http` | 1.5.0 | `MIT OR Apache-2.0` | https://github.com/hyperium/http |
| `httparse` | 1.10.1 | `MIT OR Apache-2.0` | https://github.com/seanmonstar/httparse |
| `icu_collections` | 2.4.0 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `icu_locale_core` | 2.4.0 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `icu_normalizer` | 2.4.0 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `icu_normalizer_data` | 2.4.0 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `icu_properties` | 2.4.0 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `icu_properties_data` | 2.4.0 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `icu_provider` | 2.4.0 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `idna` | 1.1.0 | `MIT OR Apache-2.0` | https://github.com/servo/rust-url/ |
| `idna_adapter` | 1.2.2 | `Apache-2.0 OR MIT` | https://github.com/hsivonen/idna_adapter |
| `image` | 0.25.10 | `MIT OR Apache-2.0` | https://github.com/image-rs/image |
| `itertools` | 0.15.0 | `MIT OR Apache-2.0` | https://github.com/rust-itertools/itertools |
| `itoa` | 1.0.19 | `MIT OR Apache-2.0` | https://github.com/dtolnay/itoa |
| `khronos_api` | 3.1.0 | `Apache-2.0` | https://github.com/brendanzab/gl-rs/ |
| `kurbo` | 0.13.1 | `Apache-2.0 OR MIT` | https://github.com/linebender/kurbo |
| `libloading` | 0.8.9 | `ISC` | https://github.com/nagisa/rust_libloading/ |
| `linebender_resource_handle` | 0.1.1 | `Apache-2.0 OR MIT` | https://github.com/linebender/raw_resource_handle |
| `litemap` | 0.8.3 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `litrs` | 1.0.1 | `MIT OR Apache-2.0` | https://github.com/LukasKalbertodt/litrs |
| `lock_api` | 0.4.14 | `MIT OR Apache-2.0` | https://github.com/Amanieu/parking_lot |
| `log` | 0.4.35 | `MIT OR Apache-2.0` | https://github.com/rust-lang/log |
| `memchr` | 2.8.3 | `Unlicense OR MIT` | https://github.com/BurntSushi/memchr |
| `memmap2` | 0.9.12 | `MIT OR Apache-2.0` | https://github.com/RazrFalcon/memmap2-rs |
| `miniz_oxide` | 0.9.1 | `MIT OR Zlib OR Apache-2.0` | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `miniz_oxide` | 0.8.9 | `MIT OR Zlib OR Apache-2.0` | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `moxcms` | 0.8.1 | `BSD-3-Clause OR Apache-2.0` | https://github.com/awxkee/moxcms.git |
| `native-tls` | 0.2.18 | `MIT OR Apache-2.0` | https://github.com/rust-native-tls/rust-native-tls |
| `nohash-hasher` | 0.2.0 | `Apache-2.0 OR MIT` | https://github.com/paritytech/nohash-hasher |
| `num-traits` | 0.2.19 | `MIT OR Apache-2.0` | https://github.com/rust-num/num-traits |
| `once_cell` | 1.21.4 | `MIT OR Apache-2.0` | https://github.com/matklad/once_cell |
| `parking_lot` | 0.12.5 | `MIT OR Apache-2.0` | https://github.com/Amanieu/parking_lot |
| `parking_lot_core` | 0.9.12 | `MIT OR Apache-2.0` | https://github.com/Amanieu/parking_lot |
| `parlance` | 0.1.1 | `Apache-2.0 OR MIT` | https://github.com/linebender/parley |
| `pem-rfc7468` | 1.0.0 | `Apache-2.0 OR MIT` | https://github.com/RustCrypto/formats |
| `peniko` | 0.6.1 | `Apache-2.0 OR MIT` | https://github.com/linebender/peniko |
| `percent-encoding` | 2.3.2 | `MIT OR Apache-2.0` | https://github.com/servo/rust-url/ |
| `pin-project-lite` | 0.2.17 | `Apache-2.0 OR MIT` | https://github.com/taiki-e/pin-project-lite |
| `png` | 0.18.1 | `MIT OR Apache-2.0` | https://github.com/image-rs/image-png |
| `poll-promise` | 0.3.0 | `MIT OR Apache-2.0` | https://github.com/EmbarkStudios/poll-promise |
| `polycool` | 0.4.0 | `MIT OR Apache-2.0` | https://github.com/linebender/kurbo |
| `potential_utf` | 0.1.6 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `proc-macro2` | 1.0.107 | `MIT OR Apache-2.0` | https://github.com/dtolnay/proc-macro2 |
| `profiling` | 1.0.18 | `MIT OR Apache-2.0` | https://github.com/aclysma/profiling |
| `pxfm` | 0.1.30 | `BSD-3-Clause OR Apache-2.0` | https://github.com/awxkee/pxfm |
| `quick-error` | 2.0.1 | `MIT/Apache-2.0` | http://github.com/tailhook/quick-error |
| `quote` | 1.0.47 | `MIT OR Apache-2.0` | https://github.com/dtolnay/quote |
| `raw-window-handle` | 0.6.2 | `MIT OR Apache-2.0 OR Zlib` | https://github.com/rust-windowing/raw-window-handle |
| `read-fonts` | 0.41.0 | `MIT OR Apache-2.0` | https://github.com/googlefonts/fontations |
| `rfd` | 0.18.0 | `MIT` | https://github.com/PolyMeilex/rfd |
| `rustls-pki-types` | 1.16.0 | `MIT OR Apache-2.0` | https://github.com/rustls/pki-types |
| `schannel` | 0.1.30 | `MIT` | https://github.com/steffengy/schannel-rs |
| `scopeguard` | 1.2.0 | `MIT OR Apache-2.0` | https://github.com/bluss/scopeguard |
| `self_cell` | 1.3.0 | `Apache-2.0 OR GPL-2.0-only` | https://github.com/Voultapher/self_cell |
| `serde` | 1.0.229 | `MIT OR Apache-2.0` | https://github.com/serde-rs/serde |
| `serde_core` | 1.0.229 | `MIT OR Apache-2.0` | https://github.com/serde-rs/serde |
| `serde_derive` | 1.0.229 | `MIT OR Apache-2.0` | https://github.com/serde-rs/serde |
| `serde_json` | 1.0.154 | `MIT OR Apache-2.0` | https://github.com/serde-rs/json |
| `simd-adler32` | 0.3.10 | `MIT` | https://github.com/mcountryman/simd-adler32 |
| `skrifa` | 0.44.0 | `MIT OR Apache-2.0` | https://github.com/googlefonts/fontations |
| `smallvec` | 1.16.5 | `MIT OR Apache-2.0` | https://github.com/servo/rust-smallvec |
| `smol_str` | 0.2.2 | `MIT OR Apache-2.0` | https://github.com/rust-analyzer/smol_str |
| `stable_deref_trait` | 1.2.1 | `MIT OR Apache-2.0` | https://github.com/storyyeller/stable_deref_trait |
| `static_assertions` | 1.1.0 | `MIT OR Apache-2.0` | https://github.com/nvzqz/static-assertions-rs |
| `syn` | 3.0.10 | `MIT OR Apache-2.0` | https://github.com/dtolnay/syn |
| `syn` | 2.0.119 | `MIT OR Apache-2.0` | https://github.com/dtolnay/syn |
| `synstructure` | 0.14.0 | `MIT` | https://github.com/mystor/synstructure |
| `sys-locale` | 0.3.2 | `MIT OR Apache-2.0` | https://github.com/1Password/sys-locale |
| `thiserror` | 2.0.21 | `MIT OR Apache-2.0` | https://github.com/dtolnay/thiserror |
| `thiserror-impl` | 2.0.21 | `MIT OR Apache-2.0` | https://github.com/dtolnay/thiserror |
| `tiff` | 0.11.3 | `MIT` | https://github.com/image-rs/image-tiff |
| `tinystr` | 0.8.5 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `tracing` | 0.1.44 | `MIT` | https://github.com/tokio-rs/tracing |
| `tracing-attributes` | 0.1.31 | `MIT` | https://github.com/tokio-rs/tracing |
| `tracing-core` | 0.1.36 | `MIT` | https://github.com/tokio-rs/tracing |
| `unicode-general-category` | 1.1.0 | `Apache-2.0` | https://github.com/yeslogic/unicode-general-category |
| `unicode-ident` | 1.0.27 | `(MIT OR Apache-2.0) AND Unicode-3.0` | https://github.com/dtolnay/unicode-ident |
| `unicode-properties` | 0.1.5 | `(MIT OR Apache-2.0) AND Unicode-3.0` | https://github.com/unicode-rs/unicode-properties |
| `unicode-script` | 0.5.9 | `MIT OR Apache-2.0` | https://github.com/unicode-rs/unicode-script |
| `unicode-segmentation` | 1.13.3 | `MIT OR Apache-2.0` | https://github.com/unicode-rs/unicode-segmentation |
| `ureq` | 3.5.2 | `MIT OR Apache-2.0` | https://github.com/algesten/ureq |
| `ureq-proto` | 0.7.1 | `MIT OR Apache-2.0` | https://github.com/algesten/ureq-proto |
| `url` | 2.5.8 | `MIT OR Apache-2.0` | https://github.com/servo/rust-url |
| `utf_types` | 1.0.0 | `Apache-2.0 OR MIT` | https://github.com/hsivonen/utf_types |
| `utf8_iter` | 1.1.0 | `Apache-2.0 OR MIT` | https://github.com/hsivonen/utf8_iter |
| `utf8-zero` | 0.8.1 | `MIT OR Apache-2.0` | https://github.com/algesten/utf8-zero |
| `uuid` | 1.29.2 | `Apache-2.0 OR MIT` | https://github.com/uuid-rs/uuid |
| `vello_common` | 0.2.0 | `Apache-2.0 OR MIT` | https://github.com/linebender/vello |
| `vello_cpu` | 0.2.0 | `Apache-2.0 OR MIT` | https://github.com/linebender/vello |
| `version_check` | 0.9.5 | `MIT/Apache-2.0` | https://github.com/SergioBenitez/version_check |
| `webbrowser` | 1.2.7 | `MIT OR Apache-2.0` | https://github.com/amodm/webbrowser-rs |
| `webpki-root-certs` | 1.0.12 | `CDLA-Permissive-2.0` | https://github.com/rustls/webpki-roots |
| `web-time` | 1.1.0 | `MIT OR Apache-2.0` | https://github.com/daxpedda/web-time |
| `weezl` | 0.1.12 | `MIT OR Apache-2.0` | https://github.com/image-rs/weezl |
| `windows` | 0.62.2 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows_x86_64_msvc` | 0.52.6 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-collections` | 0.3.2 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-core` | 0.62.2 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-future` | 0.3.2 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-implement` | 0.60.2 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-interface` | 0.59.3 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-link` | 0.2.1 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-numerics` | 0.3.1 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-result` | 0.4.1 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-strings` | 0.5.1 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.52.0 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.61.2 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-targets` | 0.52.6 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `windows-threading` | 0.2.1 | `MIT OR Apache-2.0` | https://github.com/microsoft/windows-rs |
| `winit` | 0.30.13 | `Apache-2.0` | https://github.com/rust-windowing/winit |
| `writeable` | 0.6.4 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `xml-rs` | 0.8.29 | `MIT` | https://github.com/kornelski/xml-rs |
| `yoke` | 0.8.4 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `yoke-derive` | 0.8.4 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `zerocopy` | 0.8.63 | `BSD-2-Clause OR Apache-2.0 OR MIT` | https://github.com/google/zerocopy |
| `zerocopy-derive` | 0.8.63 | `BSD-2-Clause OR Apache-2.0 OR MIT` | https://github.com/google/zerocopy |
| `zerofrom` | 0.1.8 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `zerofrom-derive` | 0.1.8 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `zeroize` | 1.9.1 | `Apache-2.0 OR MIT` | https://github.com/RustCrypto/utils |
| `zerotrie` | 0.2.5 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `zerovec` | 0.11.9 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `zerovec-derive` | 0.11.7 | `Unicode-3.0` | https://github.com/unicode-org/icu4x |
| `zlib-rs` | 0.6.8 | `Zlib` | https://github.com/trifectatechfoundation/zlib-rs |
| `zmij` | 1.0.23 | `MIT` | https://github.com/dtolnay/zmij |
| `zune-core` | 0.5.3 | `MIT OR Apache-2.0 OR Zlib` | https://github.com/etemesi254/zune-image |
| `zune-jpeg` | 0.5.15 | `MIT OR Apache-2.0 OR Zlib` | https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg |
