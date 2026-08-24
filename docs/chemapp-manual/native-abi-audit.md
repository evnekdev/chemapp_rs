# Native ABI conformance audit

## Executive summary

This audit began as a documentation-only hardening pass over all **75**
`Engine::tq...` wrappers. The hardened-audit baseline was
`ad5fbbfd71d8e5282b8c2bc63b39d193d9d678d7`. The first correction milestone
subsequently fixed the nine confirmed Win32/x86 findings described below in
`src/native.rs`. A later narrow correction also replaced the obsolete
first-space fixed-record decoding in seven getters; the historical evidence is
retained so a later platform correction does not need to rediscover it.

The strongest conclusion is deliberately scoped: the checked-in 32-bit
Windows DLL and the checked-in C bridge agree on exported decorated symbols,
stdcall stack-byte counts, argument ordering, and the non-UNIX interleaved
CHARACTER-length convention for most wrappers.  That is strong evidence for
the stated **Win32/x86/2013** build only.  It is not a certification of the
checked-in Win64 DLL or the older Linux/i386 DLL.

The earlier single-primary-verdict totals are withdrawn. They incorrectly
made a Win32 conclusion look cross-platform and also marked `TQGTHI` VERIFIED
while recording a conflicting raw length.  Status is now recorded separately
for each represented build.  At routine level, the confirmed findings are
**6 machine-ABI defects**, **1 Rust FFI-soundness declaration defect**, and
**2 semantic/API defects**. All nine are now **FIXED IN CURRENT MASTER** for
the source-level rules established by the checked Win32 bridge. `tqgetr`
remains the sole **INCOMPLETE** Win32 wrapper. The checked Linux/i386 library
also lacks **7** represented exports. Direct x64 disassembly established that
Win64 `LI`/`LIP`/`NOERR` storage is signed 32-bit and non-UNIX `LNT` is a
64-bit value. The subsequent raw-boundary correction now implements those
rules: `ChemAppInt = i32` is used for all raw INTEGER, `LIP`, and `NOERR`
pointees, while the target-specific `ChemAppLen` remains separate for
CHARACTER lengths. That correction removes the common Win64 ABI defect, but
does not promote uninspected per-routine details to verified. Unix64 has no
checked binary.

Current routine-level status is therefore: **0 current confirmed Win64
raw-integer ABI-ISSUE rows**, **1 INCOMPLETE** `TQGETR` API row on each
otherwise-supported Windows build, **74 Win64 UNVERIFIED rows**, **0 current
confirmed Win32 semantic defects**, and **7 Linux/i386
platform-unavailable rows**. The previous nine Win32 defects and the prior
common Win64 `NOERR` defect are historical fixed findings, not outstanding
issues.

**ORIGINAL AUDIT FINDING / FIXED IN CURRENT MASTER:** `Engine::tqchar`
previously declared the native `DBP` (`double *`) output as `&mut i32`, which
could write eight bytes into four-byte Rust storage. It now uses `&mut f64`
and returns `Result<f64, ChemAppError>`. The fixed-length calls (`tqgtid`,
`tqgtpi`, `tqgthi`, `tqgtrh`, `tqerr`), `tqgsu`, `tqgspc`, and `tqgpar` are
also corrected and have focused structural tests. `tqgtnm`, `tqgnsc`,
`tqgnp`, `tqmodl`, `tqgnpc`, `tqgnlc`, and `tqgsp` now preserve internal
spaces and remove only trailing fixed-record padding. The current correction
also preserves that behavior while using `ChemAppInt` for raw license outputs
such as `TQGTHI` and `TQGTED`. It does not verify uninspected Win64 routine
details or Unix64.

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
3. non-destructive export/header inspection of the checked-in libraries;
4. Win64 `llvm-objdump` disassembly of simple exports and representative
   CHARACTER-taking exports; and
5. Win64 behavioral smoke runs of `cargo run --example maindemo`.

The C files are reference evidence only.  Rust calls the exported Fortran ABI
directly, not the C functions in those files.

## Binaries and platforms actually inspected

| Build | Evidence obtained | Result |
|---|---|---|
| `windows/ca_vc_e_local.dll` | PE header: x86; timestamp in export directory 2013-10-11; `dumpbin /exports` | All 75 wrapped names are exported both as uppercase names and as `_TQ...@NN`; the `FUNCSWIN32` decorated aliases match. |
| `windows/ca_vc_e_x64.dll` | PE header: x64; timestamp 2017-11-30 02:12:47 UTC; `dumpbin /exports`; `llvm-objdump` | All 75 wrapper names are exported as undecorated uppercase `TQ...`. Disassembly establishes 32-bit `LI`/`LIP`/`NOERR` pointee storage and supports 64-bit non-UNIX `LNT` values; export names alone do not reveal those facts. |
| `linux/libLChemAppS.so` | ELF header: ELF32/i386; `objdump -T` | Lowercase trailing-underscore exports are present for the older surface.  `tqchar_`, `tqgdat_`, `tqlpar_`, `tqgpar_`, `tqcdat_`, `tqwasc_`, and `tqconf_` are absent. |
| Win32 calling convention | Win32 decorated `@NN` plus `cacint.c` non-UNIX declarations | Supports `extern "system"`/stdcall and the interleaved lengths shown below, but decoration alone is not a complete signature proof. |
| Unix/i386 calling convention | ELF symbols plus `cacint.c` UNIX declarations | Supports lowercase `_` symbols and appended `ftnlen` values.  The host is Windows, so no runtime execution was possible. |
| Win64 calling convention | export inspection, disassembly, and successful demo run | Windows x64 register/stack calling convention is used. `TQINI`, `TQVERS`, `TQLITE`, `TQNOSC`, `TQNOPC`, `TQGNP`, `TQGTED`, and the license getters show 32-bit INTEGER pointees. Representative string exports preserve the interleaved non-UNIX ordering and pass the length value in a 64-bit slot. |
| Unix64 | represented by `defs.rs` only | Not represented by a checked-in binary and not verified. |

The correction milestone repeated the Win64 smoke run after the source
changes. It initialized the library, read `data/cosi.dat`, exercised
`tqgtid`, `tqgtpi`, `tqgthi`, and `tqgsu` through the broad translated demo,
then made component/phase/condition/equilibrium/target/mapping/stream/license
calls and wrote the demo result table. The temporary `result` artifact was
removed afterward. The current raw-integer correction repeated that Win64
demo successfully, including `TQCPRT -> immediate TQERR`, all listed license
getters, file/data loading, component/phase queries, conditions,
calculations, mapping, and supported streams. It skipped optional files when
unavailable; `tqgtrh` and `tqchar` still lacked their required safe demo
conditions. A successful call sequence is **not** proof of the raw ABI.

The fixed-output conversion correction also ran this demo. `tqgtnm` returned
a non-empty license-holder value containing internal spaces and no trailing
padding; the installation-specific text was intentionally not recorded. The
current demo additionally executes `TQCPRT` followed immediately by `TQERR`;
it observed three non-empty copyright records with their record boundaries
and internal text intact. The routine now reports only structural license
facts, not local identifiers or holder text.

## Shared ABI notation

`H` below means the public C-facing declaration in `cacint.h`; `F-W` and
`F-U` mean the raw non-UNIX and UNIX declarations/calls in `cacint.c`.

- `LI`/`LIP`: on Win32 and Unix/i386 the header uses `long`/`long *` (32-bit);
  on the header's x64 branch it uses `int`/`int *` (32-bit).
- `DB`/`DBP`: `double`/`double *`.
- `LNT`: non-UNIX CHARACTER length (`long` on Win32, `size_t` in the header's
  x64 branch). On the checked x64 DLL, representative entry points preserve
  the supplied length in a 64-bit register/descriptor field, supporting the
  header's `size_t` rule. This is distinct from `LI`/`LIP`.
- `ftnlen`: UNIX CHARACTER length (`long` on Unix/i386, `int` in the header's
  x64 UNIX/Cygwin branch).
- `CMT`: `void __stdcall` on non-UNIX, `extern int` on UNIX.  The Rust code
  uses `extern "system"` on Windows and `extern "C"` on Unix.
- `C(n)`: one CHARACTER buffer with hidden/passed length `n`; `C(a,b)` means
  two buffers.  On F-W lengths are interleaved after their character buffer;
  on F-U all `ftnlen` values are appended in character-argument order.
- `I`: input, `O`: output, `IO`: in/out.  Native indices are one-based unless
  the semantic column states a documented zero/negative special case.

`native.rs` now represents every raw `LI`/`LIP`/`NOERR` pointee as signed
`ChemAppInt` (`i32`) and every raw CHARACTER length as a separate
target-specific `ChemAppLen`. Public positive `usize` indices/counts are
checked before entering the raw boundary; a negative native value is rejected
before it could become a large Rust `usize`. On checked Win64, `ChemAppLen`
is 64-bit `usize`/`size_t`; it must not be confused with `ChemAppInt`.

## Platform-status model and complete status record

The detailed inventory below records semantic and bridge evidence for every
wrapper.  The following is the authoritative platform status record; it
supersedes the old single-column primary verdict.  `VERIFIED` always means the
named build only. `ABI-ISSUE` denotes a machine declaration conflict;
`FFI-SOUNDNESS` is the separate Rust declaration class described below.

| Build | VERIFIED | ABI-ISSUE | FFI-SOUNDNESS | SEMANTICS-ISSUE | INCOMPLETE | UNVERIFIED | PLATFORM-UNAVAILABLE | NOT-REPRESENTED |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| checked Win32/x86 DLL | 74 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| checked Win64/x64 DLL | 0 | 0 | 0 | 0 | 1 | 74 | 0 | 0 |
| checked Linux/i386 SO | 0 | 0 | 0 | 0 | 0 | 68 | 7 | 0 |
| Unix64 mapping in `defs.rs` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 75 |

### Per-routine platform status

This compact matrix is exhaustive.  W32 statuses are split into the listed
exception sets and the complementary `VERIFIED` set; every other platform's
status is stated for every wrapper in its row.  `U32` is not promoted to
verified because the checked SO is older (2003) than the 2013 bridge source.

| Wrapper(s) | Win32/x86 checked DLL | Win64/x64 checked DLL | Linux/i386 checked SO | Unix64 mapping |
|---|---|---|---|---|
| `tqgtid`, `tqgtpi`, `tqgthi`, `tqgtrh`, `tqchar`, `tqerr`, `tqgspc`, `tqgsu`, `tqgpar` | VERIFIED (corrected source; checked Win32 bridge evidence) | UNVERIFIED: the common INTEGER/`NOERR` mismatch is fixed, but other details are not promoted | UNVERIFIED except unavailable `tqchar`/`tqgpar` | NOT-REPRESENTED |
| `tqgetr` | INCOMPLETE | INCOMPLETE: current scalar/unsigned API remains incomplete; other x64 details are not promoted | UNVERIFIED | NOT-REPRESENTED |
| `tqconf`, `tqgdat`, `tqlpar`, `tqcdat`, `tqwasc` | VERIFIED | UNVERIFIED: corrected raw INTEGER storage; no complete per-routine x64 verdict | PLATFORM-UNAVAILABLE | NOT-REPRESENTED |
| all remaining 60 wrappers | VERIFIED | UNVERIFIED: corrected common raw INTEGER storage; other raw details not promoted | UNVERIFIED | NOT-REPRESENTED |

The “all remaining 60” set is the 75-wrapper inventory below minus the 15
wrappers named in the preceding rows; it is intentionally a set expression,
not a claim that an unnamed platform was checked.  Together with the complete
inventory it gives one and only one status per wrapper/build.

For unambiguous machine processing and review, the same record is expanded
here one wrapper per row. Its W64 cells state the **current** post-correction
status: `U` means the common raw INTEGER defect is fixed but the routine is
not yet fully verified, and `I` retains the independent `TQGETR` API
limitation. Git retains the historical `A` status; no stale pre-correction
cell is presented as current. Key: `V` VERIFIED, `A` ABI-ISSUE,
`F` FFI-SOUNDNESS, `S` SEMANTICS-ISSUE, `I` INCOMPLETE, `U` UNVERIFIED,
`P` PLATFORM-UNAVAILABLE, `N` NOT-REPRESENTED.

| Wrapper | W32 | W64 | Linux/i386 | Unix64 |
|---|---|---|---|---|
| `tqini` | V | U | U | N |
| `tqvers` | V | U | U | N |
| `tqcprt` | V | U | U | N |
| `tqlite` | V | U | U | N |
| `tqgtid` | V | U | U | N |
| `tqgtnm` | V | U | U | N |
| `tqgtpi` | V | U | U | N |
| `tqgthi` | V | U | U | N |
| `tqgted` | V | U | U | N |
| `tqconf` | V | U | P | N |
| `tqsize` | V | U | U | N |
| `tqused` | V | U | U | N |
| `tqgio` | V | U | U | N |
| `tqcio` | V | U | U | N |
| `tqrfil` | V | U | U | N |
| `tqrbin` | V | U | U | N |
| `tqrcst` | V | U | U | N |
| `tqopen` | V | U | U | N |
| `tqwstr` | V | U | U | N |
| `tqopna` | V | U | U | N |
| `tqopnb` | V | U | U | N |
| `tqopnt` | V | U | U | N |
| `tqclos` | V | U | U | N |
| `tqgtrh` | V | U | U | N |
| `tqgsu` | V | U | U | N |
| `tqcsu` | V | U | U | N |
| `tqinsc` | V | U | U | N |
| `tqgnsc` | V | U | U | N |
| `tqcnsc` | V | U | U | N |
| `tqnosc` | V | U | U | N |
| `tqstsc` | V | U | U | N |
| `tqcsc` | V | U | U | N |
| `tqinp` | V | U | U | N |
| `tqgnp` | V | U | U | N |
| `tqmodl` | V | U | U | N |
| `tqnop` | V | U | U | N |
| `tqinpc` | V | U | U | N |
| `tqgnpc` | V | U | U | N |
| `tqpcis` | V | U | U | N |
| `tqnopc` | V | U | U | N |
| `tqstpc` | V | U | U | N |
| `tqchar` | V | U | P | N |
| `tqinlc` | V | U | U | N |
| `tqgnlc` | V | U | U | N |
| `tqnosl` | V | U | U | N |
| `tqnolc` | V | U | U | N |
| `tqgsp` | V | U | U | N |
| `tqcsp` | V | U | U | N |
| `tqgspc` | V | U | U | N |
| `tqcspc` | V | U | U | N |
| `tqsetc` | V | U | U | N |
| `tqremc` | V | U | U | N |
| `tqsttp` | V | U | U | N |
| `tqstca` | V | U | U | N |
| `tqstec` | V | U | U | N |
| `tqstrm` | V | U | U | N |
| `tqce` | V | U | U | N |
| `tqcel` | V | U | U | N |
| `tqcen` | V | U | U | N |
| `tqcenl` | V | U | U | N |
| `tqmap` | V | U | U | N |
| `tqmapl` | V | U | U | N |
| `tqclim` | V | U | U | N |
| `tqshow` | V | U | U | N |
| `tqgetr` | I | I | U | N |
| `tqgdpc` | V | U | U | N |
| `tqstxp` | V | U | U | N |
| `tqgtlc` | V | U | U | N |
| `tqbond` | V | U | U | N |
| `tqerr` | V | U | U | N |
| `tqgdat` | V | U | P | N |
| `tqlpar` | V | U | P | N |
| `tqgpar` | V | U | P | N |
| `tqcdat` | V | U | P | N |
| `tqwasc` | V | U | P | N |

## Complete routine inventory and audit matrix

The symbol column gives all represented spellings in compact form:
`W32` is the exact `defs.rs` decorated alias, `W64` is the exact uppercase
export, and `U32/U64` is the exact lower-case trailing-underscore alias.  The
Unix64 entry is represented in `defs.rs` but has no checked-in binary.
`Demo` is `C/R` for `cademo1.c`/`maindemo.rs`, `-` for neither; `crate` means
another library module calls the wrapper.  `Runtime` means the Win64 smoke
run exercised it; `indirect` means the demo's path used a helper, and `-`
means it was not exercised. The per-routine Win64 notes below are historical
evidence and must be read with the current status matrix: the common raw
INTEGER/`NOERR` defect is fixed, while remaining `UNVERIFIED` wording does not
claim the other arguments are verified.

### Initialization, licensing, files, and units (manual 2.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQINI `tqini` | §2.1; O `NOERR`; must precede normal calls; resets defaults/units. | `tqini(LIP)`; F-W/U `(LIP)`. | none; Rust `ChemAppInt*`. | `_TQINI@4` / `TQINI` / `tqini_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQVERS `tqvers` | §2.3; O version, O error; after init. | `tqvers(LIP,LIP)`; F-W/U same. | none; Rust `ChemAppInt*,ChemAppInt*`. | `_TQVERS@8` / `TQVERS` / `tqvers_`; C/R; yes. | Win32/x86: VERIFIED / — (Win32 version output is 32-bit). |
| TQCPRT `tqcprt` | §2.2; writes copyright into native message buffer. | `tqcprt(LIP)`; F-W/U same. | none; Rust `ChemAppInt*`. | `_TQCPRT@4` / `TQCPRT` / `tqcprt_`; C/R; Win64 runtime yes. | Win32/x86: VERIFIED / —. Win64 common `NOERR` defect is fixed; other raw details remain UNVERIFIED. |
| TQLITE `tqlite` | §2.4; O Light flag; phase targets/maps unavailable in Light. | `tqlite(LIP,LIP)`; F-W/U same. | none; Rust bool from `i32`. | `_TQLITE@8` / `TQLITE` / `tqlite_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQGTID `tqgtid` | §2.5; O license user ID; after init. | `tqgtid(CHP,LIP)`; F-W `(ID,255,NOERR)`, F-U `(ID,NOERR,ftnlen=255)`. | C(255); Rust now allocates and passes 255. | `_TQGTID@12` / `TQGTID` / `tqgtid_`; C/R; yes. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original hidden length was 256. Other builds UNVERIFIED. |
| TQGTNM `tqgtnm` | §2.6; O license-holder name. | `tqgtnm(CHP,LIP)`; F-W `(NAME,80,NOERR)`, F-U appended 80. | C(80); raw ABI was already correct; Rust now decodes exactly the 80-byte record. | `_TQGTNM@12` / `TQGTNM` / `tqgtnm_`; C/R; yes. | **FIXED IN CURRENT MASTER:** original `clen()` stopped at the first space. Complete internal-space-preserving license-holder text is now returned with trailing padding removed. |
| TQGTPI `tqgtpi` | §2.7; O program ID. | `tqgtpi(CHP,LIP)`; bridge passes `TQSTRLEN=25`. | C(25); Rust now allocates/passes 25. | `_TQGTPI@12` / `TQGTPI` / `tqgtpi_`; C/R; yes. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original hidden length was 80. Other builds UNVERIFIED. |
| TQGTHI `tqgthi` | §2.8; O HASP type and ID; meaningful only for relevant licensing. | `tqgthi(CHP,LIP,LIP)`; F-W `(text,25,id,noerr)`, F-U appended 25. | C(25); Rust now allocates/passes 25, with `i32*` ID. | `_TQGTHI@16` / `TQGTHI` / `tqgthi_`; C/R; yes. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original hidden length was 80. Other builds UNVERIFIED. |
| TQGTED `tqgted` | §2.9; O expiry month/year. | `tqgted(LIP,LIP,LIP)`; F-W/U same. | none; raw `ChemAppInt*`, checked public `u32` fields. | `_TQGTED@12` / `TQGTED` / `tqgted_`; C/R; yes. | Win32/x86: VERIFIED / — (non-negative fields). |
| TQCONF `tqconf` | §2.10; I option and three indices; config mutates engine. | `tqconf(CHP,LI,LI,LI,LIP)`; F-W interleaves option length, F-U appends it. | C(OPTION); checked public values become raw `ChemAppInt`. | `_TQCONF@24` / `TQCONF` / **absent**; -/-/crate; no. | Win32/x86: VERIFIED; Linux/i386: PLATFORM-UNAVAILABLE / MEDIUM. |
| TQSIZE `tqsize` | §2.11; eleven O capacity dimensions plus error; after init. | `tqsize(12×LIP)`; F-W/U same. | none; Rust twelve `ChemAppInt*`. | `_TQSIZE@48` / `TQSIZE` / `tqsize_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQUSED `tqused` | §2.12; eleven O dimensions currently used after data read. | `tqused(12×LIP)`; F-W/U same. | none; Rust eleven `i32*`. | `_TQUSED@48` / `TQUSED` / `tqused_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQGIO `tqgio` | §2.13; I option (`FILE`, `LIST`, `ERROR`, language); O unit/value; units/config dependent. | `tqgio(CHP,LIP,LIP)`; F-W `(option,len,ival,noerr)`, F-U appended length. | C(OPTION); raw output/error `ChemAppInt*`, checked public count. | `_TQGIO@16` / `TQGIO` / `tqgio_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQCIO `tqcio` | §2.14; I option and FORTRAN unit/language; mutates I/O routing; documented valid unit ranges. | `tqcio(CHP,LI,LIP)`; F-W interleaved len; F-U appended. | C(OPTION); checked public unit becomes `ChemAppInt`. | `_TQCIO@16` / `TQCIO` / `tqcio_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQRFIL `tqrfil` | §2.15; reads previously opened ASCII data file; mutates system. | `tqrfil(LIP)`; F-W/U same. | none. | `_TQRFIL@4` / `TQRFIL` / `tqrfil_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQRBIN `tqrbin` | §2.16; reads binary data; legacy/deprecated. | `tqrbin(LIP)`; F-W/U same. | none. | `_TQRBIN@4` / `TQRBIN` / `tqrbin_`; -/-/crate; no. | Win32/x86: VERIFIED / LOW: no demo coverage. |
| TQRCST `tqrcst` | §2.17; reads previously opened transparent file. | `tqrcst(LIP)`; F-W/U same. | none. | `_TQRCST@4` / `TQRCST` / `tqrcst_`; C/R/crate; optional skip. | Win32/x86: VERIFIED / —. |
| TQOPEN `tqopen` | §2.18; I filename/unit; associates a file with a FORTRAN unit. | `tqopen(CHP,LI,LIP)`; interleaved/appended length. | C(FILE); checked public unit becomes `ChemAppInt`. | `_TQOPEN@16` / `TQOPEN` / `tqopen_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQWSTR `tqwstr` | §2.19; I destination option (`LIST`/`ERROR`) and text; writes via ChemApp I/O. | `tqwstr(CHP,CHP,LIP)`; F-W interleaves both lengths; F-U appends option,text. | C(OPTION,TEXT). | `_TQWSTR@20` / `TQWSTR` / `tqwstr_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQOPNA `tqopna` | §2.20; I ASCII filename/unit; precedes TQRFIL. | `tqopna(CHP,LI,LIP)`; interleaved/appended length. | C(FILE). | `_TQOPNA@16` / `TQOPNA` / `tqopna_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQOPNB `tqopnb` | §2.21; I binary filename/unit; precedes TQRBIN. | `tqopnb(CHP,LI,LIP)`; interleaved/appended length. | C(FILE). | `_TQOPNB@16` / `TQOPNB` / `tqopnb_`; -/-/crate; no. | Win32/x86: VERIFIED / LOW. |
| TQOPNT `tqopnt` | §2.22; I transparent filename/unit; precedes TQRCST. | `tqopnt(CHP,LI,LIP)`; interleaved/appended length. | C(FILE). | `_TQOPNT@16` / `TQOPNT` / `tqopnt_`; C/R/crate; optional skip. | Win32/x86: VERIFIED / —. |
| TQCLOS `tqclos` | §2.23; I unit; closes ChemApp-associated file. | `tqclos(LI,LIP)`; F-W/U same. | none; checked public unit becomes `ChemAppInt`. | `_TQCLOS@8` / `TQCLOS` / `tqclos_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQGTRH `tqgtrh` | §2.24; ten O header fields after TQRCST: version, names, version/date arrays, ID, user, remark. | Header form shown in `cacint.h`; F-W interleaves 40,40,255,80,80; F-U appends them. | C(40,40,255,80,80); Rust now allocates/passes those exact lengths. | `_TQGTRH@64` / `TQGTRH` / `tqgtrh_`; C/R; optional skip. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original values were 41,41,256,81,81. Other builds UNVERIFIED. |
| TQGSU `tqgsu` | §2.25; I unit class; O active unit; units are mutable engine state. | `tqgsu(CHP,CHP,LIP)`; F-W option length/unit length interleaved; F-U appended. | C(option,25); Rust now uses the `CString` byte length, matching `strlen(OPTION)`. | `_TQGSU@20` / `TQGSU` / `tqgsu_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original code truncated options and underflowed when empty. Other builds UNVERIFIED. |
| TQCSU `tqcsu` | §2.26; I unit class and unit string; changes active system units. | `tqcsu(CHP,CHP,LIP)`; F-W interleaved, F-U appended. | C(option,unit). | `_TQCSU@20` / `TQCSU` / `tqcsu_`; C/R; yes. | Win32/x86: VERIFIED / —. |

### System identity, status, and sublattices (manual 3.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQINSC `tqinsc` | §3.2; I component name, O one-based index; ASCII system loaded. | `(CHP,LIP,LIP)`; interleaved/appended name length. | C(NAME). | `_TQINSC@16` / `TQINSC` / `tqinsc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQGNSC `tqgnsc` | §3.3; I one-based component index, O name. | `(LI,CHP,LIP)`; name length 25. | C(25); Rust decodes the declared fixed record. | `_TQGNSC@16` / `TQGNSC` / `tqgnsc_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** replaces first-space truncation; raw ABI unchanged. |
| TQCNSC `tqcnsc` | §3.4; I component index/name; changes name. | `(LI,CHP,LIP)`; input name length. | C(NAME). | `_TQCNSC@16` / `TQCNSC` / `tqcnsc_`; -/-/-; no. | Win32/x86: VERIFIED / LOW. |
| TQNOSC `tqnosc` | §3.5; O number of system components. | `(LIP,LIP)`. | none. | `_TQNOSC@8` / `TQNOSC` / `tqnosc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQSTSC `tqstsc` | §3.6; I component index; O stoichiometry vector, molecular mass in current amount unit/mol. | `(LI,DBP,DBP,LIP)`. | array `DB*`; Rust allocates `TQNOSC` values. | `_TQSTSC@16` / `TQSTSC` / `tqstsc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQCSC `tqcsc` | §3.7; I complete component-name set; must be independent; mutates component basis. | `(CHP,LIP)`; bridge packs C rows into blank-padded 24-byte records, raw length 24. | C array(24); Rust makes packed 24-byte records. | `_TQCSC@12` / `TQCSC` / `tqcsc_`; C/R; yes. | Win32/x86: VERIFIED / —; unusual packed buffer matches raw, not public C input. |
| TQINP `tqinp` | §3.8; I phase name, O one-based phase index. | `(CHP,LIP,LIP)`. | C(NAME). | `_TQINP@16` / `TQINP` / `tqinp_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQGNP `tqgnp` | §3.9; I phase index, O name. | `(LI,CHP,LIP)`; output len 25. | C(25); Rust decodes the declared fixed record. | `_TQGNP@16` / `TQGNP` / `tqgnp_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** replaces first-space truncation; raw ABI unchanged. |
| TQMODL `tqmodl` | §3.10; I phase index, O model identifier. | `(LI,CHP,LIP)`; output len 25. | C(25); Rust decodes the declared fixed record. | `_TQMODL@16` / `TQMODL` / `tqmodl_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** consistent record-bounded output conversion. |
| TQNOP `tqnop` | §3.11; O number of phases. | `(LIP,LIP)`. | none. | `_TQNOP@8` / `TQNOP` / `tqnop_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQINPC `tqinpc` | §3.12; I name/phase index, O one-based constituent index. | `(CHP,LI,LIP,LIP)`. | C(NAME). | `_TQINPC@20` / `TQINPC` / `tqinpc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQGNPC `tqgnpc` | §3.13; I phase/constituent index, O name. | `(LI,LI,CHP,LIP)`, output len 25. | C(25); Rust decodes the declared fixed record. | `_TQGNPC@20` / `TQGNPC` / `tqgnpc_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** trailing padding is removed without changing internal spaces. |
| TQPCIS `tqpcis` | §3.14; I phase/constituent, O permitted-as-incoming flag. | `(LI,LI,LIP,LIP)`. | none. | `_TQPCIS@16` / `TQPCIS` / `tqpcis_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQNOPC `tqnopc` | §3.15; I phase, O number of constituents. | `(LI,LIP,LIP)`. | none. | `_TQNOPC@12` / `TQNOPC` / `tqnopc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQSTPC `tqstpc` | §3.16; I phase/constituent; O stoichiometry and molecular mass, active-unit dependent. | `(LI,LI,DBP,DBP,LIP)`. | DB array; Rust allocates component count. | `_TQSTPC@20` / `TQSTPC` / `tqstpc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQCHAR `tqchar` | §3.17; I phase/constituent; O charge as real value. | Header/bridge `(LI,LI,DBP,LIP)`. | no CHAR; Rust now uses `f64*`, matching raw `double*`. | `_TQCHAR@16` / `TQCHAR` / **absent**; -/-/crate; no. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original `i32*` was a CRITICAL eight-byte-write risk. Linux/i386 unavailable; other builds UNVERIFIED. |
| TQINLC `tqinlc` | §3.18; I name/phase/sublattice; O constituent index. | `(CHP,LI,LI,LIP,LIP)`. | C(NAME). | `_TQINLC@24` / `TQINLC` / `tqinlc_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQGNLC `tqgnlc` | §3.19; I phase/sublattice/constituent; O name. | `(LI,LI,LI,CHP,LIP)`; output len 25. | C(25); Rust decodes the declared fixed record. | `_TQGNLC@24` / `TQGNLC` / `tqgnlc_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** consistent record-bounded output conversion. |
| TQNOSL `tqnosl` | §3.20; I phase; O number of sublattices. | `(LI,LIP,LIP)`. | none. | `_TQNOSL@12` / `TQNOSL` / `tqnosl_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQNOLC `tqnolc` | §3.21; I phase/sublattice; O constituent count. | `(LI,LI,LIP,LIP)`. | none. | `_TQNOLC@16` / `TQNOLC` / `tqnolc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQGSP `tqgsp` | §3.23; I phase; O status (`ENTERED`, `ELIMINATED`, etc.). | `(LI,CHP,LIP)`, output len 25. | C(25); Rust decodes the declared fixed record. | `_TQGSP@16` / `TQGSP` / `tqgsp_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** trailing fixed-width padding is removed. |
| TQCSP `tqcsp` | §3.24; I phase/status; changes phase participation. | `(LI,CHP,LIP)`. | C(STATUS). | `_TQCSP@16` / `TQCSP` / `tqcsp_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQGSPC `tqgspc` | §3.25; I phase/constituent; O status. | `(LI,LI,CHP,LIP)`, output len 25. | C(25); Rust symbol type now uses `&mut u8` for the writable output pointer. | `_TQGSPC@20` / `TQGSPC` / `tqgspc_`; C/R/crate; yes. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original immutable reference was an FFI-soundness defect, not a pointer-layout change. Other builds UNVERIFIED. |
| TQCSPC `tqcspc` | §3.26; I phase/constituent/status; mutates status subject to model restrictions. | `(LI,LI,CHP,LIP)`. | C(STATUS). | `_TQCSPC@20` / `TQCSPC` / `tqcspc_`; C/R; yes. | Win32/x86: VERIFIED / —. |

### Conditions and streams (manual 4.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQSETC `tqsetc` | §4.1; I option/indexP/index/value; O condition number.  `INDEXP/INDEX`: component, phase, constituent or system per documented zero rules; conditions use active units.  Cannot mix with stream amounts. | `(CHP,LI,LI,DB,LIP,LIP)`; F-W interleaves option len, F-U appends. | C(OPTION), `f64*`; checked public indices and raw output use `ChemAppInt`. | `_TQSETC@28` / `TQSETC` / `tqsetc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —; native low-level method preserves one-based convention. |
| TQREMC `tqremc` | §4.2; I condition number; `0`, `-1`, `-2` have documented reset meanings; `-2` preserves units. | `(LI,LIP)`. | none; Rust correctly uses `i32` for negative specials. | `_TQREMC@8` / `TQREMC` / `tqremc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQSTTP `tqsttp` | §4.3; I stream identifier and two-element T/P vector; creates/sets stream. | `(CHP,DBP,LIP)`. | C(IDENTS); `f64[2]`. | `_TQSTTP@16` / `TQSTTP` / `tqsttp_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQSTCA `tqstca` | §4.4; I stream ID, phase/constituent, amount; stream workflow only; active amount unit. | `(CHP,LI,LI,DB,LIP)`. | C(IDENTS). | `_TQSTCA@24` / `TQSTCA` / `tqstca_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQSTEC `tqstec` | §4.5; I option, phase, value; stream target/global condition semantics; active units. | `(CHP,LI,DB,LIP)`. | C(OPTION). | `_TQSTEC@20` / `TQSTEC` / `tqstec_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQSTRM `tqstrm` | §4.6; I stream identifier; removes stream. | `(CHP,LIP)`. | C(IDENTS). | `_TQSTRM@12` / `TQSTRM` / `tqstrm_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |

### Calculation and result retrieval (manual 5.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQCE `tqce` | §5.1; I target option/indexes/two limits; normal equilibrium ignores target arguments; calculates/mutates current result. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION); `f64[2]`. | `_TQCE@24` / `TQCE` / `tqce_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQCEL `tqcel` | §5.2; same as TQCE, additionally emits result table to LIST. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQCEL@24` / `TQCEL` / `tqcel_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQCEN `tqcen` | §5.3; recalculates using prior equilibrium estimates; requires prior successful TQCE/TQCEL. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQCEN@24` / `TQCEN` / `tqcen_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQCENL `tqcenl` | §5.4; TQCEN plus LIST table. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQCENL@24` / `TQCENL` / `tqcenl_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQMAP `tqmap` | §5.5; I first/next map option, indexes, interval; O continuation; results are stateful and must be captured before next call; unavailable in Light. | `(CHP,LI,LI,DBP,LIP,LIP)`. | C(OPTION); `f64[2]`. | `_TQMAP@28` / `TQMAP` / `tqmap_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQMAPL `tqmapl` | §5.6; TQMAP plus table output. | `(CHP,LI,LI,DBP,LIP,LIP)`. | C(OPTION). | `_TQMAPL@28` / `TQMAPL` / `tqmapl_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQCLIM `tqclim` | §5.7; I option/value; alters target/map bounds; active units apply. | `(CHP,DB,LIP)`. | C(OPTION). | `_TQCLIM@16` / `TQCLIM` / `tqclim_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQSHOW `tqshow` | §5.8; writes current state/settings to LIST; no calculation. | `(LIP)`. | none. | `_TQSHOW@4` / `TQSHOW` / `tqshow_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQGETR `tqgetr` | §5.9; I result option/indexes, O scalar or array from current result only. `INDEXP>0/INDEX<0` selects all constituents (or `XP`/`AP` system components) of one phase; `INDEXP<0/INDEX=0` all phases; `INDEXP<=0/INDEX<0` all system components. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION); Rust exposes one `f64` and `usize` indices only. | `_TQGETR@24` / `TQGETR` / `tqgetr_`; C/R/crate; yes. | Win32/x86: INCOMPLETE / MEDIUM: cannot represent negative documented indices or safely receive the array forms. |
| TQGDPC `tqgdpc` | §5.10; I property option/phase/constituent, O value; documented dimensionless/unit rules depend on option and active units. | `(CHP,LI,LI,DBP,LIP)`. | C(OPTION). | `_TQGDPC@24` / `TQGDPC` / `tqgdpc_`; C/R; yes. | Win32/x86: VERIFIED / —. |
| TQSTXP `tqstxp` | §5.11; I stream ID/property option, O property; stream state/units apply. | `(CHP,CHP,DBP,LIP)`. | C(IDENTS,OPTION), F-U appends both lengths. | `_TQSTXP@24` / `TQSTXP` / `tqstxp_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQGTLC `tqgtlc` | §5.12; I phase/sublattice/constituent, O current calculated site fraction. | `(LI,LI,LI,DBP,LIP)`. | none. | `_TQGTLC@20` / `TQGTLC` / `tqgtlc_`; C/R/crate; yes. | Win32/x86: VERIFIED / —. |
| TQBOND `tqbond` | §5.13; I phase and pair/quadruplet indexes, O current fraction; applicable models only. | `(LI,LI,LI,LI,LI,DBP,LIP)`. | none. | `_TQBOND@28` / `TQBOND` / `tqbond_`; -/-/crate; no. | Win32/x86: VERIFIED / LOW. |
| TQERR `tqerr` | §5.14; O current three-line message; must be checked close to origin. | `(CHP,LIP)`; bridge raw calls length **80**, with a 3×80 buffer. | C(80) record length; Rust now retains 240-byte storage but passes 80 and joins trimmed records. | `_TQERR@12` / `TQERR` / `tqerr_`; C/R (commented Rust); no. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED; original hidden length was 240. Other builds UNVERIFIED. |

### Thermodynamic data manipulation (manual 6.x)

| Routine / wrapper | Manual semantic arguments, state, units | H and reconstructed raw ABI | CHAR / Rust ABI types | Symbols; coverage; runtime | Verdict / severity / audit note |
|---|---|---|---|---|---|
| TQGDAT `tqgdat` | §6.1; I phase/constituent/option/range; O count and value vector; ASCII data required; options define vector size. | `(LI,LI,CHP,LI,LIP,DBP,LIP)`. | C(OPTION); Rust fixed `[f64;25]`. | `_TQGDAT@32` / `TQGDAT` / **absent**; -/-/crate; no. | Win32/x86: VERIFIED; Linux/i386: PLATFORM-UNAVAILABLE / MEDIUM: fixed capacity still needs option-by-option bounds proof. |
| TQLPAR `tqlpar` | §6.2; I phase/option; O parameter count, text records, lengths; ASCII/model dependent. | `(LI,CHP,LIP,CHP,LIP,LIP)`, text record len 156. | C(OPTION,156); Rust 1999×156, ignores returned lengths. | `_TQLPAR@32` / `TQLPAR` / **absent**; -/-/crate; no. | Win32/x86: VERIFIED; Linux/i386: PLATFORM-UNAVAILABLE / MEDIUM: lossy record-length handling noted. |
| TQGPAR `tqgpar` | §6.3; I phase/option/index; O expression/value counts and values; ASCII/model dependent. | `(LI,CHP,LI,LIP,LIP,DBP,LIP)`. | C(OPTION); Rust fixed 28×20 and now returns through `wrap_result`. | `_TQGPAR@32` / `TQGPAR` / **absent**; -/-/crate; no. | **FIXED IN CURRENT MASTER:** Win32/x86 VERIFIED for error propagation; fixed output capacities remain a later bounds audit. Linux/i386: PLATFORM-UNAVAILABLE. |
| TQCDAT `tqcdat` | §6.4; five I integer selectors and I value; changes ASCII thermodynamic data. | `(LI,LI,LI,LI,LI,DB,LIP)`. | none. | `_TQCDAT@28` / `TQCDAT` / **absent**; -/-/crate; no. | Win32/x86: VERIFIED; Linux/i386: PLATFORM-UNAVAILABLE / MEDIUM. |
| TQWASC `tqwasc` | §6.5; I output filename; writes ASCII data where capability permits. | `(CHP,LIP)` plus file length. | C(FILE). | `_TQWASC@12` / `TQWASC` / **absent**; -/-/-; no. | Win32/x86: VERIFIED; Linux/i386: PLATFORM-UNAVAILABLE / MEDIUM. |

## Separate semantic findings

1. **FIXED IN CURRENT MASTER:** `tqgsu` now derives its input length from
   `CString::as_bytes()`, matching the bridge's `strlen(OPTION)` and allowing
   an empty option without arithmetic underflow. Win64 direct-binary evidence
   supports its 64-bit interleaved length values; Unix64 remains unverified.
2. `tqgetr` at `src/native.rs` takes `usize` indices, allocates one
   `f64`, and returns one scalar, but manual §5.9 requires `VAL` to be an
   array for `(INDEXP>0, INDEX<0)` (all constituents of a phase, or all
   system components of a phase for `XP`/`AP`), `(INDEXP<0, INDEX=0)` (all
   phases), and `(INDEXP<=0, INDEX<0)` (all system components).  Negative
   values are therefore unrepresentable and the one-element allocation would
   be unsafe if the native array form were made reachable.  The high-level
   demo retrieves one fugacity at a time and does not expose the gap.
   Severity: MEDIUM / INCOMPLETE API.
3. **FIXED IN CURRENT MASTER:** `tqgpar` now calls `wrap_result(vecc,
   errcode)`. Its documented output-capacity bounds remain a separate audit.
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
| checked Win64 | immediately after each explicit CHARACTER argument | 64-bit by-value `LNT`/`size_t` is supported by representative x64 disassembly; do not confuse this with 32-bit `LI`/`LIP` |
| Unix64 | project has mappings, but no checked binary | unverified; do not infer from pointer width |

### Complete CHARACTER revalidation

This is a re-check of all **50** character-taking wrappers. For the Win32
bridge, `W:` means every length immediately follows its character pointer;
for UNIX/i386, `U:` means all lengths are appended, in explicit-character
order. The Rust declarations use `usize` for both length forms. That width
matches the checked Win32 stack evidence and is now supported for Win64
non-UNIX `LNT`; Unix64 remains unverified.

| Character wrapper(s) | Direction and native declared length | Rust buffer / length actually passed | Result |
|---|---|---|---|
| `tqconf`, `tqgio`, `tqcio`, `tqopen`, `tqopna`, `tqopnb`, `tqopnt`, `tqwasc` | one input option/file; W: after it, U: final; `strlen(input)` | `CString`, `str.len()` excluding terminator; no blank padding | Win32 length value/order matches bridge. |
| `tqinsc`, `tqcnsc`, `tqinp`, `tqinpc`, `tqinlc` | one input name; W interleaved/U appended; `strlen(input)` | `CString`, `str.len()` | Win32 match. |
| `tqcsp`, `tqcspc`, `tqsetc`, `tqsttp`, `tqstca`, `tqstec`, `tqstrm`, `tqce`, `tqcel`, `tqcen`, `tqcenl`, `tqmap`, `tqmapl`, `tqclim`, `tqgetr`, `tqgdpc`, `tqgdat`, `tqlpar`, `tqgpar` | one input status/identifier/option; W interleaved/U appended; `strlen(input)` | `CString`, `str.len()` | Win32 match. `tqgetr` remains incomplete; `tqgpar` capacity bounds remain a separate audit. |
| `tqcsc` | input 2-D character records; W length after pointer/U final; fixed 24 per record, bridge blank-pads C rows | Rust packs and space-pads 24-byte records, passes 24 | Win32 match; this unusual packing correctly follows raw ABI rather than public C shape. |
| `tqwstr`, `tqstxp` | two inputs; W lengths interleaved in argument order/U both appended in argument order; `strlen` for each | two `CString`s; lengths use the corresponding `str.len()` | Win32 order/value match. |
| `tqcsu` | two inputs (class, unit), W interleaved/U appended; both `strlen` | two `CString`s; both `str.len()` | Win32 match. |
| `tqgsu` | input option (`strlen`) and output unit (fixed 25); W `(option,len,unit,25,noerr)`, U appends `(strlen,25)` | `CString` byte length for option; mutable 25-byte unit buffer, passes 25 | **FIXED IN CURRENT MASTER:** Win32 length/order match; original code used `option.len()-1`. |
| `tqgtid` | fixed output ID, 255; W `(id,255,noerr)`, U appended 255 | mutable 255-byte buffer; hidden length 255 | **FIXED IN CURRENT MASTER:** Win32 length/order match; original hidden length was 256. |
| `tqgtnm` | fixed output name, 80 | mutable 80-byte buffer; hidden 80 | **FIXED IN CURRENT MASTER:** raw ABI was already correct; `clen()` first-space truncation was replaced with record-bounded decoding. |
| `tqgtpi`, `tqgthi` | each fixed output, `TQSTRLEN` = 25 | mutable 25-byte buffers; hidden 25 | **FIXED IN CURRENT MASTER:** Win32 length/order match; original hidden length was 80. |
| `tqgnsc`, `tqgnp`, `tqmodl`, `tqgnpc`, `tqgnlc`, `tqgsp` | fixed output name/model/status, 25 | mutable 25-byte buffer; hidden 25 | **FIXED IN CURRENT MASTER:** all use the same record-bounded conversion; this is Rust-side output adaptation, not a machine ABI change. |
| `tqgspc` | fixed writable output status, 25 | mutable 25-byte allocation, hidden 25, and `Symbol` uses `&mut u8` | **FIXED IN CURRENT MASTER:** Win32 soundness declaration now expresses the native write. |
| `tqgtrh` | five fixed outputs 40, 40, 255, 80, 80; W interleaves each/U appends in that order | exact allocations and hidden values 40, 40, 255, 80, 80 | **FIXED IN CURRENT MASTER:** Win32 length/order match; original values were off by one. |
| `tqerr` | output is three 80-byte records; W `(mess,80,noerr)`, U final 80 | 240-byte allocation, hidden 80, record-bounded conversion | **FIXED IN CURRENT MASTER:** total capacity remains distinct from CHARACTER record length. |

Rust passes pointers to the first byte of `CString::as_bytes()` for inputs, so
the NUL terminator is present in the allocation but excluded from the native
length. The bridge uses C `strlen` for those inputs; it does not blank-pad
them. Fixed outputs are Fortran blank-padded, are not required to be NUL
terminated, and must be interpreted using their declared record size. The
corrected fixed-output wrappers inspect only that declared record, stop at an
in-record NUL when present, and trim trailing Fortran blanks without removing
internal spaces. The previous statement that Win32/x86 already had 74 fully
verified wrappers was therefore overstated while `tqgtnm`'s first-space
conversion remained. After this correction, 74 is the current Win32/x86 count:
the seven output-adaptation defects are fixed and `tqgetr` remains incomplete.

## Win64 direct-binary evidence (2026-08-24)

The examined file was `windows/ca_vc_e_x64.dll`, PE x64, image base
`0x180000000`, timestamp `2017-11-30 02:12:47 UTC`. The reproducible command
used `C:\\gcc64\\bin\\llvm-objdump.exe -d` with the following entry addresses:
`TQINI` `0x180008cc0`, `TQCPRT` `0x180009030`, `TQVERS` `0x180009220`,
`TQLITE` `0x1800092d0`, `TQGIO` `0x18000a5a0`, `TQGSU` `0x18000aa00`,
`TQCSU` `0x18000abe0`, `TQNOSC` `0x18000c9c0`, `TQGNP` `0x18000d7f0`,
`TQNOP` `0x18000da00`, `TQNOPC` `0x18000e220`, `TQWSTR` `0x180009940`,
`TQGTID` `0x180048280`, `TQGTED` `0x180048660`, `TQGTHI` `0x180048d80`,
`TQGTNM` `0x1800494e0`, `TQGTPI` `0x18004a780`, and `TQGTRH`
`0x18004e620`.

| Question | Evidence and conclusion |
|---|---|
| Native INTEGER / `LI` / `LIP` | **32-bit signed storage.** `TQINI` zeroes `NOERR` with `movl`; `TQVERS` writes both version and `NOERR` with `movl`; `TQNOSC` and `TQNOPC` use 32-bit loads/stores for counts and indices; `TQGNP` uses 32-bit arithmetic through its index pointer; `TQGTED` writes its date outputs with `movl`. This agrees with the x64 `cacint.h` `int`/`int *` branch. |
| `NOERR` | **32-bit signed storage.** `TQINI`, `TQCPRT`, `TQVERS`, `TQLITE`, all inspected license getters, and `TQGSU` write exactly a DWORD through the error pointer. The current source uses `ChemAppInt` for every raw `NOERR`; this removes the formerly common Win64 declaration conflict. |
| CHARACTER length / `LNT` | **64-bit by-value slot, supported by direct evidence.** `TQGTID`, `TQGTNM`, `TQGTPI`, `TQGTHI`, `TQGIO`, `TQGSU`, `TQCSU`, and `TQWSTR` preserve the supplied length in 64-bit registers and descriptor fields before use. This matches the x64 header's `LNT size_t`; it is not evidence that `LI` is pointer-sized. |
| CHARACTER placement | **Interleaved non-UNIX ordering.** `TQGNP` receives `(index*, buffer, length, noerr*)` in `RCX/RDX/R8/R9`; `TQGSU` receives `(option, option_length, unit, unit_length, noerr*)`, with the fifth pointer on the Win64 stack; `TQCSU`/`TQWSTR` preserve the same adjacent-length order. `TQGTRH` consumes the corresponding register/stack argument slots consistently with the bridge's five-pair order; all of its per-field values remain governed by the recorded bridge evidence. |
| Calling / return convention | **Windows x64 / `extern "system"`.** The exports are undecorated jump stubs to ordinary x64 bodies and return with `retq`; scalar results are written via pointer arguments. The manual calls them subroutines and no selected routine establishes a meaningful return register value, so Rust `-> ()` remains the correct non-observing declaration. |

Representative instruction-level observations are deliberately compact:

- `TQINI`/`TQCPRT` clear `NOERR` through a 32-bit (`movl`/DWORD) memory
  destination, rather than a QWORD store.
- `TQNOSC` and `TQNOPC` use 32-bit count/index loads and stores; `TQGNP`
  consumes its index through a 32-bit operation. These are independent
  examples of output count and input index treatment.
- For `TQGNP`, the `(index*, buffer, length, noerr*)` Win64 slots place the
  length in the full 64-bit third argument register (`R8`) while `NOERR` is a
  pointer in the fourth (`R9`). This demonstrates why a 64-bit `ChemAppLen`
  and a 32-bit pointee `ChemAppInt` coexist in one call.

This does not make every non-integer parameter or every routine semantically
verified. It establishes and the current source implements a common raw type
rule: on Win64, raw `LI`/`LIP`/`NOERR` use explicit `ChemAppInt` (`i32`)
storage, while raw non-UNIX CHARACTER lengths use `ChemAppLen`
(`usize`/`size_t`) value arguments. Public `usize` indices are checked before
conversion to raw `i32`; negative native values are rejected by positive-index
adapters so future APIs can preserve documented sentinel semantics.

## Integer width and calling-convention findings

On the verified Win32 source/binary, `usize` has the same *width* as the
bridge's 32-bit `LI`/`LIP`; all Win32 `@NN` aliases in `defs.rs` exactly match
the library, including `_TQERR@12`.  `f64` matches `DB`; `i32` matches the
Win32 32-bit integer storage used for dimensions and IDs where applied.

The Win64 header branch defines `LI int` and `LIP int *`, not 64-bit `long`
or pointer-sized integers. The direct 2017 DLL evidence above now confirms
that rule for representative input, output, count, index, and `NOERR`
pointers. `native.rs` now uses `ChemAppInt` for all raw INTEGER pointer
arguments and `NOERR`, with checked public-value conversion at the boundary.
The correction is based on the direct-binary evidence, not merely the
successful demo; uninspected per-routine ABI facts remain UNVERIFIED.

The Linux library is ELF32/i386, so its exported trailing-underscore names
support `FUNCSUNIX32`, not `FUNCSUNIX64`.  It is older than the full wrapper
surface; resolving later symbols will fail cleanly through `libloading` rather
than proving a signature.

### UNIX return convention: resolved scope, unresolved provenance

`cacint.h` defines `CMT` as `extern int` under `UNIX`, and the raw declarations
in `cacint.c` consequently use a C `int` result for every listed routine.
The manual presents these routines as FORTRAN `CALL` subroutines and the C
bridge never reads a return value.  On 32-bit System V cdecl, an integer return
uses `EAX` and does not change the caller-cleaned argument layout; Rust's
`extern "C" fn(...) -> ()` therefore makes the same call and merely ignores
that register.  This is **not a demonstrated parameter, stack, or hidden-
length ABI mismatch** for the Unix declarations.

The available evidence cannot establish whether the older i386 binary
intentionally returns a meaningful integer, leaves `EAX` unspecified as a
FORTRAN-subroutine artefact, or follows an old C-interface convention.  The
return *value* is consequently UNVERIFIED and must not be used.  This does not
by itself change a Rust call to `-> ()` into a proven defect; the broader U32
rows remain UNVERIFIED because no version-matched bridge/runtime conformance
evidence was available.

## License and interface conformance

`TQERR` retrieves ChemApp's current internal message buffer, not an immutable
per-call result. The conformance sequence is therefore exactly
`TQINI -> TQCPRT -> immediate TQERR`, matching `cademo1.c`; intervening calls
must not be inserted before reading the three `CHARACTER*80` records. The
Rust wrapper allocates 240 bytes, passes the native per-record length 80, and
converts the three records independently before joining non-empty records
with exactly one newline. A pure regression test uses synthetic copyright-like
text to prove that record boundaries, internal spaces, a long second record,
and trailing blank removal are all preserved without committing vendor text.

The checked Win64 `maindemo` run successfully initialized the x64 DLL,
executed that sequence, and observed a structurally complete three-record
copyright result. It also successfully exercised `TQVERS`, `TQLITE`,
`TQGTID`, `TQGTNM`, `TQGTPI`, `TQGTHI`, and `TQGTED`. `TQGTNM` was non-empty,
contained internal spaces, and had no trailing fixed-record padding. The
demo intentionally reports only those facts, never the local holder name,
user ID, dongle ID, token, or absolute path.

No direct C-vs-Rust record comparison was committed for this pass. The host
is Windows and the repository has no matching Win64 C import library/build
path for the older transition source; the checked Linux reference is ELF32
i386 and cannot run under this host's Windows loader. No Linux/i386 C
reference executable was therefore built or executed. These environment
limits do not change the Rust runtime result or establish a Linux ABI result.
A controlled non-copyright native-error probe was intentionally deferred: it
would need a documented harmless option and an isolated engine state.

## C demo versus Rust translation

`cademo1.c` invokes **64** distinct native routines. `maindemo.rs` now has
active corresponding invocation expressions for the same **64** routine
names, including the canonical `TQINI`, `TQCPRT`, immediate `TQERR`, and
`TQVERS` opening sequence. Optional `subl-ex.dat`/`cosiex.cst` paths remain
skippable when their files are absent. The translation otherwise mirrors the
important order:
query FILE unit; open/read/close; unit changes; global conditions; TQCE/TQCEN;
target and mapping continuation; streams; sublattices; and transparent-file
metadata.

The eleven wrappers not exercised by either broad demo are `tqconf`, `tqrbin`,
`tqopnb`, `tqcnsc`, `tqchar`, `tqbond`, `tqgdat`, `tqlpar`, `tqgpar`,
`tqcdat`, and `tqwasc`.  Several are used elsewhere in the
crate, especially cache/entity code.  `entitiesdemo.rs` is a small
Calculator/entity example and is not a translation of the C demo or evidence
of comprehensive native coverage.

## Unknowns and recommended correction order

1. **FIXED IN CURRENT MASTER:** `tqchar`, the fixed CHARACTER calls, `tqgsu`,
   `tqgpar`, `tqgspc`, fixed-record decoding, and the common Win64 raw
   INTEGER/`NOERR` mismatch have focused structural coverage. `ChemAppInt`
   and `ChemAppLen` remain explicitly separate.
2. Split `tqgetr` into scalar and correctly sized array forms, retaining its
   documented negative indices; bound-check data-manipulation buffers.
3. Add capability/symbol detection for the older Linux/i386 binary, then
   decide its supported minimum ChemApp version.  Obtain a real Unix64 binary
   before claiming Unix64 support.
4. Separately repair higher-level loader unit selection, error swallowing,
   mapping continuation, and state/unit documentation.

The recommended next production milestone is the **TQGETR signed-selector +
scalar/array API redesign**. It must preserve the now-correct signed raw
INTEGER representation rather than introduce another unsigned ABI boundary.
