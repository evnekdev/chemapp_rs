# Native ABI conformance audit

## Executive summary

This is a documentation-only audit of all **75** `Engine::tq...` wrappers in
`src/native.rs` at commit `2227e9210f98a298a4c23e16bd2b4322c55c2c02`.
No FFI declaration, reference source, proprietary binary, data-file, or demo
was changed by this audit.

The strongest conclusion is deliberately scoped: the checked-in 32-bit
Windows DLL and the checked-in C bridge agree on exported decorated symbols,
stdcall stack-byte counts, argument ordering, and the non-UNIX interleaved
CHARACTER-length convention for most wrappers.  That is strong evidence for
the stated **Win32/x86/2013** build only.  It is not a certification of the
checked-in Win64 DLL or the older Linux/i386 DLL.

Primary verdict counts (one per wrapper) are: **61 VERIFIED**, **6 ABI-ISSUE**,
**1 SEMANTICS-ISSUE**, **6 PLATFORM-SPECIFIC**, **1 INCOMPLETE**, and
**0 UNVERIFIED**.  `VERIFIED` means verified for the explicitly stated
Win32/x86 build; the cross-platform caveats below still apply.

There is one CRITICAL finding: `Engine::tqchar` declares the native output as
`&mut i32`, but the matching C header and bridge pass a `DBP` (`double *`).
The native routine can write eight bytes into a four-byte Rust object.  This
is a likely memory-corruption bug on the verified Win32 ABI.  Do not call it
until a separate correction task fixes and tests it.

High findings are the fixed-length mismatches in `tqgtid`, `tqgtpi`,
`tqgtrh`, and `tqerr`, and the output pointer mutability mismatch in
`tqgspc`.  The Windows x64 `LI`/`LIP` question is also HIGH priority, but is
recorded as a build-specific *unverified risk*, not as an established raw-ABI
conflict, because the available C bridge is from 2013 and the checked-in x64
DLL is from 2017.

## Scope, methodology, and evidence boundary

Semantic authority was the official Programmer's Manual, Edition 4.00, dated
23 March 2016, for ChemApp 2.0.2--6.4.0:

- [manual index](https://gtt-technologies.de/ca-doc/index.html);
- chapter 2 for initialization, I/O, files and units;
- chapter 3 for system data and status;
- chapter 4 for global/stream conditions;
- chapter 5 for calculation/results;
- chapter 6 for data manipulation; and
- chapters 1.6, 1.10 and 7 for state, best practice, Light restrictions, and
  errors.

Raw-ABI evidence was kept separate from manual semantics:

1. `examples/cacint.h`, revision 2571, 2014-05-14;
2. `examples/cacint.c`, revision 2499, 2013-09-25;
3. non-destructive export/header inspection of the checked-in libraries; and
4. one Win64 behavioral smoke run of `cargo run --example maindemo`.

The C files are reference evidence only.  Rust calls the exported Fortran ABI
directly, not the C functions in those files.

## Binaries and platforms actually inspected

| Build | Evidence obtained | Result |
|---|---|---|
| `windows/ca_vc_e_local.dll` | PE header: x86; timestamp in export directory 2013-10-11; `dumpbin /exports` | All 75 wrapped names are exported both as uppercase names and as `_TQ...@NN`; the `FUNCSWIN32` decorated aliases match. |
| `windows/ca_vc_e_x64.dll` | PE header: x64; `dumpbin /exports` | All 75 wrapper names are exported as undecorated uppercase `TQ...`.  Export names do not reveal integer or hidden-length widths. |
| `linux/libLChemAppS.so` | ELF header: ELF32/i386; `objdump -T` | Lowercase trailing-underscore exports are present for the older surface.  `tqchar_`, `tqgdat_`, `tqlpar_`, `tqgpar_`, `tqcdat_`, `tqwasc_`, and `tqconf_` are absent. |
| Win32 calling convention | Win32 decorated `@NN` plus `cacint.c` non-UNIX declarations | Supports `extern "system"`/stdcall and the interleaved lengths shown below, but decoration alone is not a complete signature proof. |
| Unix/i386 calling convention | ELF symbols plus `cacint.c` UNIX declarations | Supports lowercase `_` symbols and appended `ftnlen` values.  The host is Windows, so no runtime execution was possible. |
| Win64 calling convention | export inspection plus successful demo run | `extern "system"` loads/runs for the exercised path.  Widths and raw string-length types remain unverified against a version-matched bridge or disassembly/conformance harness. |
| Unix64 | represented by `defs.rs` only | Not represented by a checked-in binary and not verified. |

The Win64 smoke run initialized the library, read `data/cosi.dat`, made
component/phase/condition/equilibrium/target/mapping/stream/license calls,
and wrote the demo result table.  The temporary `result` artifact was removed
afterward.  It skipped optional files when unavailable.  A successful call
sequence is **not** proof of the raw ABI.

## Shared ABI notation

`H` below means the public C-facing declaration in `cacint.h`; `F-W` and
`F-U` mean the raw non-UNIX and UNIX declarations/calls in `cacint.c`.

- `LI`/`LIP`: on Win32 and Unix/i386 the header uses `long`/`long *` (32-bit);
  on the header's x64 branch it uses `int`/`int *` (32-bit).
- `DB`/`DBP`: `double`/`double *`.
- `LNT`: non-UNIX CHARACTER length (`long` on Win32, `size_t` in the header's
  x64 branch).  The bridge declarations/calls spell the supplied values as
  `int`; this discrepancy itself needs version-matched Win64 confirmation.
- `ftnlen`: UNIX CHARACTER length (`long` on Unix/i386, `int` in the header's
  x64 UNIX/Cygwin branch).
- `CMT`: `void __stdcall` on non-UNIX, `extern int` on UNIX.  The Rust code
  uses `extern "system"` on Windows and `extern "C"` on Unix.
- `C(n)`: one CHARACTER buffer with hidden/passed length `n`; `C(a,b)` means
  two buffers.  On F-W lengths are interleaved after their character buffer;
  on F-U all `ftnlen` values are appended in character-argument order.
- `I`: input, `O`: output, `IO`: in/out.  Native indices are one-based unless
  the semantic column states a documented zero/negative special case.

All `native.rs` wrappers use `usize` for `NOERR` and most native integer
arguments.  That has the right storage width for the verified Win32 and
Unix/i386 source evidence, but is not automatically correct on Win64: the
header's x64 `LIP` is `int *`, while `&mut usize` points to eight bytes.  The
low four bytes happen to make common small values work on little-endian x64;
that is not sufficient ABI proof.

## Complete routine inventory and audit matrix

The symbol column gives all represented spellings in compact form:
`W32` is the exact `defs.rs` decorated alias, `W64` is the exact uppercase
export, and `U32/U64` is the exact lower-case trailing-underscore alias.  The
Unix64 entry is represented in `defs.rs` but has no checked-in binary.
`Demo` is `C/R` for `cademo1.c`/`maindemo.rs`, `-` for neither; `crate` means
another library module calls the wrapper.  `Runtime` means the Win64 smoke
run exercised it; `indirect` means the demo's path used a helper, and `-`
means it was not exercised.

### Initialization, licensing, files, and units (manual 2.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQINI `tqini` | §2.1; O `NOERR`; must precede normal calls; resets defaults/units. | `tqini(LIP)`; F-W/U `(LIP)`. | none; Rust `usize*`. | `_TQINI@4` / `TQINI` / `tqini_`; C/R/crate; yes. | VERIFIED / —. |
| TQVERS `tqvers` | §2.3; O version, O error; after init. | `tqvers(LIP,LIP)`; F-W/U same. | none; Rust `i32*,usize*`. | `_TQVERS@8` / `TQVERS` / `tqvers_`; C/R; yes. | VERIFIED / — (Win32 version output is 32-bit). |
| TQCPRT `tqcprt` | §2.2; writes copyright into native message buffer. | `tqcprt(LIP)`; F-W/U same. | none. | `_TQCPRT@4` / `TQCPRT` / `tqcprt_`; C/R (commented in Rust); no. | VERIFIED / —. |
| TQLITE `tqlite` | §2.4; O Light flag; phase targets/maps unavailable in Light. | `tqlite(LIP,LIP)`; F-W/U same. | none; Rust bool from `i32`. | `_TQLITE@8` / `TQLITE` / `tqlite_`; C/R; yes. | VERIFIED / —. |
| TQGTID `tqgtid` | §2.5; O license user ID; after init. | `tqgtid(CHP,LIP)`; F-W `(ID,255,NOERR)`, F-U `(ID,NOERR,ftnlen=255)`. | C(255); Rust sends 256. | `_TQGTID@12` / `TQGTID` / `tqgtid_`; C/R; yes. | ABI-ISSUE / HIGH: raw length is 255, not 256. |
| TQGTNM `tqgtnm` | §2.6; O license-holder name. | `tqgtnm(CHP,LIP)`; F-W `(NAME,80,NOERR)`, F-U appended 80. | C(80); Rust `u8[80]`. | `_TQGTNM@12` / `TQGTNM` / `tqgtnm_`; C/R; yes. | VERIFIED / —. |
| TQGTPI `tqgtpi` | §2.7; O program ID. | `tqgtpi(CHP,LIP)`; bridge passes `TQSTRLEN=25`. | C(25); Rust `u8[80]`, sends 80. | `_TQGTPI@12` / `TQGTPI` / `tqgtpi_`; C/R; yes. | ABI-ISSUE / HIGH: length differs from bridge. |
| TQGTHI `tqgthi` | §2.8; O HASP type and ID; meaningful only for relevant licensing. | `tqgthi(CHP,LIP,LIP)`; F-W `(text,25,id,noerr)`, F-U appended 25. | C(25); `i32*` ID. | `_TQGTHI@16` / `TQGTHI` / `tqgthi_`; C/R; yes. | VERIFIED / —; changelog correction agrees with bridge. |
| TQGTED `tqgted` | §2.9; O expiry month/year. | `tqgted(LIP,LIP,LIP)`; F-W/U same. | none; Rust `u32*`. | `_TQGTED@12` / `TQGTED` / `tqgted_`; C/R; yes. | VERIFIED / — (non-negative fields). |
| TQCONF `tqconf` | §2.10; I option and three indices; config mutates engine. | `tqconf(CHP,LI,LI,LI,LIP)`; F-W interleaves option length, F-U appends it. | C(OPTION); Rust `usize*` values. | `_TQCONF@24` / `TQCONF` / **absent**; -/-/crate; no. | PLATFORM-SPECIFIC / MEDIUM: absent from checked Linux/i386 library. |
| TQSIZE `tqsize` | §2.11; eleven O capacity dimensions plus error; after init. | `tqsize(12×LIP)`; F-W/U same. | none; Rust eleven `i32*`, `usize*` error. | `_TQSIZE@48` / `TQSIZE` / `tqsize_`; C/R; yes. | VERIFIED / —. |
| TQUSED `tqused` | §2.12; eleven O dimensions currently used after data read. | `tqused(12×LIP)`; F-W/U same. | none; Rust eleven `i32*`. | `_TQUSED@48` / `TQUSED` / `tqused_`; C/R; yes. | VERIFIED / —. |
| TQGIO `tqgio` | §2.13; I option (`FILE`, `LIST`, `ERROR`, language); O unit/value; units/config dependent. | `tqgio(CHP,LIP,LIP)`; F-W `(option,len,ival,noerr)`, F-U appended length. | C(OPTION); Rust `usize*`. | `_TQGIO@16` / `TQGIO` / `tqgio_`; C/R/crate; yes. | VERIFIED / —. |
| TQCIO `tqcio` | §2.14; I option and FORTRAN unit/language; mutates I/O routing; documented valid unit ranges. | `tqcio(CHP,LI,LIP)`; F-W interleaved len; F-U appended. | C(OPTION); Rust `usize*`. | `_TQCIO@16` / `TQCIO` / `tqcio_`; C/R/crate; yes. | VERIFIED / —. |
| TQRFIL `tqrfil` | §2.15; reads previously opened ASCII data file; mutates system. | `tqrfil(LIP)`; F-W/U same. | none. | `_TQRFIL@4` / `TQRFIL` / `tqrfil_`; C/R/crate; yes. | VERIFIED / —. |
| TQRBIN `tqrbin` | §2.16; reads binary data; legacy/deprecated. | `tqrbin(LIP)`; F-W/U same. | none. | `_TQRBIN@4` / `TQRBIN` / `tqrbin_`; -/-/crate; no. | VERIFIED / LOW: no demo coverage. |
| TQRCST `tqrcst` | §2.17; reads previously opened transparent file. | `tqrcst(LIP)`; F-W/U same. | none. | `_TQRCST@4` / `TQRCST` / `tqrcst_`; C/R/crate; optional skip. | VERIFIED / —. |
| TQOPEN `tqopen` | §2.18; I filename/unit; associates a file with a FORTRAN unit. | `tqopen(CHP,LI,LIP)`; interleaved/appended length. | C(FILE); Rust `usize*`. | `_TQOPEN@16` / `TQOPEN` / `tqopen_`; C/R/crate; yes. | VERIFIED / —. |
| TQWSTR `tqwstr` | §2.19; I destination option (`LIST`/`ERROR`) and text; writes via ChemApp I/O. | `tqwstr(CHP,CHP,LIP)`; F-W interleaves both lengths; F-U appends option,text. | C(OPTION,TEXT). | `_TQWSTR@20` / `TQWSTR` / `tqwstr_`; C/R; yes. | VERIFIED / —. |
| TQOPNA `tqopna` | §2.20; I ASCII filename/unit; precedes TQRFIL. | `tqopna(CHP,LI,LIP)`; interleaved/appended length. | C(FILE). | `_TQOPNA@16` / `TQOPNA` / `tqopna_`; C/R/crate; yes. | VERIFIED / —. |
| TQOPNB `tqopnb` | §2.21; I binary filename/unit; precedes TQRBIN. | `tqopnb(CHP,LI,LIP)`; interleaved/appended length. | C(FILE). | `_TQOPNB@16` / `TQOPNB` / `tqopnb_`; -/-/crate; no. | VERIFIED / LOW. |
| TQOPNT `tqopnt` | §2.22; I transparent filename/unit; precedes TQRCST. | `tqopnt(CHP,LI,LIP)`; interleaved/appended length. | C(FILE). | `_TQOPNT@16` / `TQOPNT` / `tqopnt_`; C/R/crate; optional skip. | VERIFIED / —. |
| TQCLOS `tqclos` | §2.23; I unit; closes ChemApp-associated file. | `tqclos(LI,LIP)`; F-W/U same. | none; Rust `usize*`. | `_TQCLOS@8` / `TQCLOS` / `tqclos_`; C/R/crate; yes. | VERIFIED / —. |
| TQGTRH `tqgtrh` | §2.24; ten O header fields after TQRCST: version, names, version/date arrays, ID, user, remark. | Header form shown in `cacint.h`; F-W interleaves 40,40,255,80,80; F-U appends them. | C(40,40,255,80,80); Rust sends 41,41,256,81,81. | `_TQGTRH@64` / `TQGTRH` / `tqgtrh_`; C/R; optional skip. | ABI-ISSUE / HIGH: all five raw lengths are off by one. |
| TQGSU `tqgsu` | §2.25; I unit class; O active unit; units are mutable engine state. | `tqgsu(CHP,CHP,LIP)`; F-W option length/unit length interleaved; F-U appended. | C(option,25); Rust computes `option.len()-1`. | `_TQGSU@20` / `TQGSU` / `tqgsu_`; C/R/crate; yes. | SEMANTICS-ISSUE / HIGH: passes a truncated option and underflows for empty input. |
| TQCSU `tqcsu` | §2.26; I unit class and unit string; changes active system units. | `tqcsu(CHP,CHP,LIP)`; F-W interleaved, F-U appended. | C(option,unit). | `_TQCSU@20` / `TQCSU` / `tqcsu_`; C/R; yes. | VERIFIED / —. |

### System identity, status, and sublattices (manual 3.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQINSC `tqinsc` | §3.2; I component name, O one-based index; ASCII system loaded. | `(CHP,LIP,LIP)`; interleaved/appended name length. | C(NAME). | `_TQINSC@16` / `TQINSC` / `tqinsc_`; C/R/crate; yes. | VERIFIED / —. |
| TQGNSC `tqgnsc` | §3.3; I one-based component index, O name. | `(LI,CHP,LIP)`; name length 25. | C(25). | `_TQGNSC@16` / `TQGNSC` / `tqgnsc_`; C/R/crate; yes. | VERIFIED / —. |
| TQCNSC `tqcnsc` | §3.4; I component index/name; changes name. | `(LI,CHP,LIP)`; input name length. | C(NAME). | `_TQCNSC@16` / `TQCNSC` / `tqcnsc_`; -/-/-; no. | VERIFIED / LOW. |
| TQNOSC `tqnosc` | §3.5; O number of system components. | `(LIP,LIP)`. | none. | `_TQNOSC@8` / `TQNOSC` / `tqnosc_`; C/R/crate; yes. | VERIFIED / —. |
| TQSTSC `tqstsc` | §3.6; I component index; O stoichiometry vector, molecular mass in current amount unit/mol. | `(LI,DBP,DBP,LIP)`. | array `DB*`; Rust allocates `TQNOSC` values. | `_TQSTSC@16` / `TQSTSC` / `tqstsc_`; C/R/crate; yes. | VERIFIED / —. |
| TQCSC `tqcsc` | §3.7; I complete component-name set; must be independent; mutates component basis. | `(CHP,LIP)`; bridge packs C rows into blank-padded 24-byte records, raw length 24. | C array(24); Rust makes packed 24-byte records. | `_TQCSC@12` / `TQCSC` / `tqcsc_`; C/R; yes. | VERIFIED / —; unusual packed buffer matches raw, not public C input. |
| TQINP `tqinp` | §3.8; I phase name, O one-based phase index. | `(CHP,LIP,LIP)`. | C(NAME). | `_TQINP@16` / `TQINP` / `tqinp_`; C/R/crate; yes. | VERIFIED / —. |
| TQGNP `tqgnp` | §3.9; I phase index, O name. | `(LI,CHP,LIP)`; output len 25. | C(25). | `_TQGNP@16` / `TQGNP` / `tqgnp_`; C/R/crate; yes. | VERIFIED / —. |
| TQMODL `tqmodl` | §3.10; I phase index, O model identifier. | `(LI,CHP,LIP)`; output len 25. | C(25). | `_TQMODL@16` / `TQMODL` / `tqmodl_`; C/R/crate; yes. | VERIFIED / —. |
| TQNOP `tqnop` | §3.11; O number of phases. | `(LIP,LIP)`. | none. | `_TQNOP@8` / `TQNOP` / `tqnop_`; C/R/crate; yes. | VERIFIED / —. |
| TQINPC `tqinpc` | §3.12; I name/phase index, O one-based constituent index. | `(CHP,LI,LIP,LIP)`. | C(NAME). | `_TQINPC@20` / `TQINPC` / `tqinpc_`; C/R/crate; yes. | VERIFIED / —. |
| TQGNPC `tqgnpc` | §3.13; I phase/constituent index, O name. | `(LI,LI,CHP,LIP)`, output len 25. | C(25). | `_TQGNPC@20` / `TQGNPC` / `tqgnpc_`; C/R/crate; yes. | VERIFIED / LOW: output is not trimmed consistently. |
| TQPCIS `tqpcis` | §3.14; I phase/constituent, O permitted-as-incoming flag. | `(LI,LI,LIP,LIP)`. | none. | `_TQPCIS@16` / `TQPCIS` / `tqpcis_`; C/R/crate; yes. | VERIFIED / —. |
| TQNOPC `tqnopc` | §3.15; I phase, O number of constituents. | `(LI,LIP,LIP)`. | none. | `_TQNOPC@12` / `TQNOPC` / `tqnopc_`; C/R/crate; yes. | VERIFIED / —. |
| TQSTPC `tqstpc` | §3.16; I phase/constituent; O stoichiometry and molecular mass, active-unit dependent. | `(LI,LI,DBP,DBP,LIP)`. | DB array; Rust allocates component count. | `_TQSTPC@20` / `TQSTPC` / `tqstpc_`; C/R/crate; yes. | VERIFIED / —. |
| TQCHAR `tqchar` | §3.17; I phase/constituent; O charge as real value. | Header/bridge `(LI,LI,DBP,LIP)`. | no CHAR; Rust uses `i32*` where raw is `double*`. | `_TQCHAR@16` / `TQCHAR` / **absent**; -/-/crate; no. | ABI-ISSUE / **CRITICAL**: likely eight-byte native write into four-byte object. |
| TQINLC `tqinlc` | §3.18; I name/phase/sublattice; O constituent index. | `(CHP,LI,LI,LIP,LIP)`. | C(NAME). | `_TQINLC@24` / `TQINLC` / `tqinlc_`; C/R; yes. | VERIFIED / —. |
| TQGNLC `tqgnlc` | §3.19; I phase/sublattice/constituent; O name. | `(LI,LI,LI,CHP,LIP)`; output len 25. | C(25). | `_TQGNLC@24` / `TQGNLC` / `tqgnlc_`; C/R/crate; yes. | VERIFIED / —. |
| TQNOSL `tqnosl` | §3.20; I phase; O number of sublattices. | `(LI,LIP,LIP)`. | none. | `_TQNOSL@12` / `TQNOSL` / `tqnosl_`; C/R; yes. | VERIFIED / —. |
| TQNOLC `tqnolc` | §3.21; I phase/sublattice; O constituent count. | `(LI,LI,LIP,LIP)`. | none. | `_TQNOLC@16` / `TQNOLC` / `tqnolc_`; C/R/crate; yes. | VERIFIED / —. |
| TQGSP `tqgsp` | §3.23; I phase; O status (`ENTERED`, `ELIMINATED`, etc.). | `(LI,CHP,LIP)`, output len 25. | C(25). | `_TQGSP@16` / `TQGSP` / `tqgsp_`; C/R/crate; yes. | VERIFIED / LOW: returned padding retained. |
| TQCSP `tqcsp` | §3.24; I phase/status; changes phase participation. | `(LI,CHP,LIP)`. | C(STATUS). | `_TQCSP@16` / `TQCSP` / `tqcsp_`; C/R; yes. | VERIFIED / —. |
| TQGSPC `tqgspc` | §3.25; I phase/constituent; O status. | `(LI,LI,CHP,LIP)`, output len 25. | C(25); Rust symbol type says `&u8`, not mutable output pointer. | `_TQGSPC@20` / `TQGSPC` / `tqgspc_`; C/R/crate; yes. | ABI-ISSUE / HIGH: raw output must be mutable. |
| TQCSPC `tqcspc` | §3.26; I phase/constituent/status; mutates status subject to model restrictions. | `(LI,LI,CHP,LIP)`. | C(STATUS). | `_TQCSPC@20` / `TQCSPC` / `tqcspc_`; C/R; yes. | VERIFIED / —. |

### Conditions and streams (manual 4.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQSETC `tqsetc` | §4.1; I option/indexP/index/value; O condition number.  `INDEXP/INDEX`: component, phase, constituent or system per documented zero rules; conditions use active units.  Cannot mix with stream amounts. | `(CHP,LI,LI,DB,LIP,LIP)`; F-W interleaves option len, F-U appends. | C(OPTION), `f64*`; Rust `usize*` indices. | `_TQSETC@28` / `TQSETC` / `tqsetc_`; C/R/crate; yes. | VERIFIED / —; native low-level method preserves one-based convention. |
| TQREMC `tqremc` | §4.2; I condition number; `0`, `-1`, `-2` have documented reset meanings; `-2` preserves units. | `(LI,LIP)`. | none; Rust correctly uses `i32` for negative specials. | `_TQREMC@8` / `TQREMC` / `tqremc_`; C/R/crate; yes. | VERIFIED / —. |
| TQSTTP `tqsttp` | §4.3; I stream identifier and two-element T/P vector; creates/sets stream. | `(CHP,DBP,LIP)`. | C(IDENTS); `f64[2]`. | `_TQSTTP@16` / `TQSTTP` / `tqsttp_`; C/R; yes. | VERIFIED / —. |
| TQSTCA `tqstca` | §4.4; I stream ID, phase/constituent, amount; stream workflow only; active amount unit. | `(CHP,LI,LI,DB,LIP)`. | C(IDENTS). | `_TQSTCA@24` / `TQSTCA` / `tqstca_`; C/R/crate; yes. | VERIFIED / —. |
| TQSTEC `tqstec` | §4.5; I option, phase, value; stream target/global condition semantics; active units. | `(CHP,LI,DB,LIP)`. | C(OPTION). | `_TQSTEC@20` / `TQSTEC` / `tqstec_`; C/R; yes. | VERIFIED / —. |
| TQSTRM `tqstrm` | §4.6; I stream identifier; removes stream. | `(CHP,LIP)`. | C(IDENTS). | `_TQSTRM@12` / `TQSTRM` / `tqstrm_`; C/R/crate; yes. | VERIFIED / —. |

### Calculation and result retrieval (manual 5.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQCE `tqce` | §5.1; I target option/indexes/two limits; normal equilibrium ignores target arguments; calculates/mutates current result. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION); `f64[2]`. | `_TQCE@24` / `TQCE` / `tqce_`; C/R/crate; yes. | VERIFIED / —. |
| TQCEL `tqcel` | §5.2; same as TQCE, additionally emits result table to LIST. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQCEL@24` / `TQCEL` / `tqcel_`; C/R; yes. | VERIFIED / —. |
| TQCEN `tqcen` | §5.3; recalculates using prior equilibrium estimates; requires prior successful TQCE/TQCEL. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQCEN@24` / `TQCEN` / `tqcen_`; C/R; yes. | VERIFIED / —. |
| TQCENL `tqcenl` | §5.4; TQCEN plus LIST table. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQCENL@24` / `TQCENL` / `tqcenl_`; C/R; yes. | VERIFIED / —. |
| TQMAP `tqmap` | §5.5; I first/next map option, indexes, interval; O continuation; results are stateful and must be captured before next call; unavailable in Light. | `(CHP,LI,LI,DBP,LIP,LIP)`. | C(OPTION); `f64[2]`. | `_TQMAP@28` / `TQMAP` / `tqmap_`; C/R/crate; yes. | VERIFIED / —. |
| TQMAPL `tqmapl` | §5.6; TQMAP plus table output. | `(CHP,LI,LI,DBP,LIP,LIP)`. | C(OPTION). | `_TQMAPL@28` / `TQMAPL` / `tqmapl_`; C/R/crate; yes. | VERIFIED / —. |
| TQCLIM `tqclim` | §5.7; I option/value; alters target/map bounds; active units apply. | `(CHP,DB,LIP)`. | C(OPTION). | `_TQCLIM@16` / `TQCLIM` / `tqclim_`; C/R/crate; yes. | VERIFIED / —. |
| TQSHOW `tqshow` | §5.8; writes current state/settings to LIST; no calculation. | `(LIP)`. | none. | `_TQSHOW@4` / `TQSHOW` / `tqshow_`; C/R/crate; yes. | VERIFIED / —. |
| TQGETR `tqgetr` | §5.9; I result option/indexes, O scalar **or documented array** from current result only; zero/negative indexes have option-dependent meanings. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION); Rust exposes one `f64` only. | `_TQGETR@24` / `TQGETR` / `tqgetr_`; C/R/crate; yes. | INCOMPLETE / MEDIUM: cannot safely expose documented array results (for example `INDEX=-1`). |
| TQGDPC `tqgdpc` | §5.10; I property option/phase/constituent, O value; documented dimensionless/unit rules depend on option and active units. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQGDPC@24` / `TQGDPC` / `tqgdpc_`; C/R; yes. | VERIFIED / —. |
| TQSTXP `tqstxp` | §5.11; I stream ID/property option, O property; stream state/units apply. | `(CHP,CHP,DBP,LIP)`. | C(IDENTS,OPTION), F-U appends both lengths. | `_TQSTXP@24` / `TQSTXP` / `tqstxp_`; C/R/crate; yes. | VERIFIED / —. |
| TQGTLC `tqgtlc` | §5.12; I phase/sublattice/constituent, O current calculated site fraction. | `(LI,LI,LI,DBP,LIP)`. | none. | `_TQGTLC@20` / `TQGTLC` / `tqgtlc_`; C/R/crate; yes. | VERIFIED / —. |
| TQBOND `tqbond` | §5.13; I phase and pair/quadruplet indexes, O current fraction; applicable models only. | `(LI,LI,LI,LI,LI,DBP,LIP)`. | none. | `_TQBOND@28` / `TQBOND` / `tqbond_`; -/-/crate; no. | VERIFIED / LOW. |
| TQERR `tqerr` | §5.14; O current three-line message; must be checked close to origin. | `(CHP,LIP)`; bridge raw calls length **80**, with a 3×80 buffer. | C(80) record length; Rust passes 240. | `_TQERR@12` / `TQERR` / `tqerr_`; C/R (commented Rust); no. | ABI-ISSUE / HIGH: raw CHARACTER length must be 80, not total buffer size. |

### Thermodynamic data manipulation (manual 6.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQGDAT `tqgdat` | §6.1; I phase/constituent/option/range; O count and value vector; ASCII data required; options define vector size. | `(LI,LI,CHP,LI,LIP,DBP,LIP)`. | C(OPTION); Rust fixed `[f64;25]`. | `_TQGDAT@32` / `TQGDAT` / **absent**; -/-/crate; no. | PLATFORM-SPECIFIC / HIGH: absent Linux/i386; separately, fixed capacity requires option-by-option bounds proof. |
| TQLPAR `tqlpar` | §6.2; I phase/option; O parameter count, text records, lengths; ASCII/model dependent. | `(LI,CHP,LIP,CHP,LIP,LIP)`, text record len 156. | C(OPTION,156); Rust 1999×156, ignores returned lengths. | `_TQLPAR@32` / `TQLPAR` / **absent**; -/-/crate; no. | PLATFORM-SPECIFIC / MEDIUM: absent Linux/i386; lossy record-length handling noted. |
| TQGPAR `tqgpar` | §6.3; I phase/option/index; O expression/value counts and values; ASCII/model dependent. | `(LI,CHP,LI,LIP,LIP,DBP,LIP)`. | C(OPTION); Rust fixed 28×20 and returns `Ok` without checking `errcode`. | `_TQGPAR@32` / `TQGPAR` / **absent**; -/-/crate; no. | PLATFORM-SPECIFIC / HIGH: absent Linux/i386; error swallowing is a separate semantic defect. |
| TQCDAT `tqcdat` | §6.4; five I integer selectors and I value; changes ASCII thermodynamic data. | `(LI,LI,LI,LI,LI,DB,LIP)`. | none. | `_TQCDAT@28` / `TQCDAT` / **absent**; -/-/crate; no. | PLATFORM-SPECIFIC / MEDIUM. |
| TQWASC `tqwasc` | §6.5; I output filename; writes ASCII data where capability permits. | `(CHP,LIP)` plus file length. | C(FILE). | `_TQWASC@12` / `TQWASC` / **absent**; -/-/-; no. | PLATFORM-SPECIFIC / MEDIUM. |

## Separate semantic findings

1. `tqgsu` at `src/native.rs:695` sends `option.len() - 1`.  The bridge uses
   the actual `strlen(OPTION)`.  This truncates every non-empty option and
   panics/underflows for empty input.  Severity: HIGH.
2. `tqgetr` at `src/native.rs:1685` returns a scalar only, although its
   documented `DBP VAL` output can be an array for option/index combinations.
   The high-level demo retrieves one fugacity at a time and therefore does not
   expose the gap.  Severity: MEDIUM.
3. `tqgpar` at `src/native.rs:1886` builds the return value and uses
   `return Ok(vecc)` rather than `wrap_result(vecc, errcode)`.  A native
   error is silently discarded.  Severity: HIGH.
4. `Calculator::load_datafile` hard-codes unit 10 rather than obtaining the
   configured `FILE` unit through TQGIO.  `Calculator` mapping helpers do not
   exhaust continuation results, and some entity/cache code is deliberately
   lossy/experimental.  These are higher-level workflow findings, not proof
   of an `Engine` raw-ABI defect.
5. Entity accessors and iterators in `src/entities/` and `src/iterator/`
   commonly replace native failures with `NaN`, `false`, `<NONE>`, or zero.
   This conflicts with the manual's check-every-error guidance.  The snapshot
   layer correctly copies live entities before subsequent calculations, but it
   copies those lossy values if a query fails.
6. `src/entities/stream.rs` defines an inherent `fn drop` rather than an
   `impl Drop for Stream`; it is therefore not automatically invoked and does
   not remove the native stream on Rust drop.  The stream's documented units
   are also only comments, not an established unit policy.
7. The cache/parse layers use the data-manipulation surface with fixed
   assumptions and several `unwrap()`/`todo!()` paths.  They should remain
   classified as experimental until their ASCII-file/model preconditions and
   parameter bounds are independently tested.

## Separate ABI and CHARACTER findings

The bridge has one raw rule per represented ABI:

| Platform family | Raw string-length position | Length type evidence |
|---|---|---|
| non-UNIX / checked Win32 | immediately after each explicit CHARACTER argument | `LNT`/the bridge's `int` call values; Win32 decoration corroborates each total stack size |
| UNIX / checked Linux i386 | appended after all explicit arguments, in CHARACTER argument order | `ftnlen`, which is `long` on i386 in `cacint.h` |
| Win64 and Unix64 | project has mappings, but no matching raw bridge/binary pair | unverified; do not infer from pointer width |

String wrappers whose length position/order matches the bridge include TQGIO,
TQCIO, TQGSU/TQCSU (apart from the `tqgsu` value bug), all name/status lookup
routines, streams, equilibrium/map/result option routines, TQWSTR,
TQSTXP, TQLPAR and TQGTRH's *position*.  The known length-value conflicts
are TQGTID, TQGTPI, TQGTRH and TQERR.

The bridge's output behavior is also material: it blank-pads fixed Fortran
strings, then removes trailing spaces in its C wrapper.  `native.rs` sometimes
trims (`tqmodl`, `tqgnlc`) and sometimes returns the full fixed buffer
(`tqgnpc`, `tqgsp`, `tqgspc`, `tqerr`).  That is not necessarily raw ABI
wrong, but it is inconsistent Rust conversion behavior.

## Integer width and calling-convention findings

On the verified Win32 source/binary, `usize` has the same *width* as the
bridge's 32-bit `LI`/`LIP`; all Win32 `@NN` aliases in `defs.rs` exactly match
the library, including `_TQERR@12`.  `f64` matches `DB`; `i32` matches the
Win32 32-bit integer storage used for dimensions and IDs where applied.

The Win64 header branch defines `LI int` and `LIP int *`, not 64-bit `long`
or pointer-sized integers.  `native.rs` passes most integers and all errors
as `usize`.  Since no version-matched 2017 bridge or ABI-level Win64 test
exists, this audit does **not** promote that concern to a confirmed ABI issue;
it is the first correction/research milestone.  In particular, never use
successful x64 calls as justification for treating `usize` as ChemApp's
integer type.

The Linux library is ELF32/i386, so its exported trailing-underscore names
support `FUNCSUNIX32`, not `FUNCSUNIX64`.  It is older than the full wrapper
surface; resolving later symbols will fail cleanly through `libloading` rather
than proving a signature.

## C demo versus Rust translation

`cademo1.c` uses 64 distinct native routines.  `maindemo.rs` uses the same 64
distinct routines, with two intentional differences: copyright/message calls
are commented out, and optional `subl-ex.dat`/`cosiex.cst` paths are skipped
when the files are absent.  The translation mirrors the C demo's important
order: query FILE unit; open/read/close; unit changes; global conditions;
TQCE/TQCEN; target and mapping continuation; streams; sublattices; and
transparent-file metadata.

The eleven wrappers not exercised by either broad demo are `tqconf`, `tqrbin`,
`tqopnb`, `tqcnsc`, `tqchar`, `tqbond`, `tqgdat`, `tqlpar`, `tqgpar`,
`tqcdat`, and `tqwasc`.  Several are used elsewhere in the
crate, especially cache/entity code.  `entitiesdemo.rs` is a small
Calculator/entity example and is not a translation of the C demo or evidence
of comprehensive native coverage.

## Unknowns and recommended correction order

1. **Stop/fix/test `tqchar` first.**  Correct its output to an ABI-proven
   double representation and add a focused Win32 conformance test.
2. Correct the four fixed CHARACTER-length calls and the `tqgspc` mutable
   output declaration; test exact output lengths and blank trimming.
3. Establish the actual Win64 Fortran integer/length ABI from a version-
   matched GTT transition source, compiler documentation, disassembly, or a
   narrowly scoped conformance harness.  Replace `usize` only with evidence.
4. Split/clarify `tqgetr` array results and make `tqgpar` propagate NOERR;
   bound-check the data-manipulation buffers.
5. Add capability/symbol detection for the older Linux/i386 binary, then
   decide its supported minimum ChemApp version.  Obtain a real Unix64 binary
   before claiming Unix64 support.
6. Separately repair higher-level loader unit selection, error swallowing,
   mapping continuation, and state/unit documentation.

No correction in this list was made by this audit.
