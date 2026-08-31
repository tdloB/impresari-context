# YARA Engine Selection Evaluation

- Status: Complete; Option A (YARA-X) selected
- Observation date: 2026-08-31
- Product baseline: `0c4ec6d86e0e16b2a46ac094c39e030ae9fa4993`
- Decision record: [ADR-0097](../decisions/0097-reconsider-first-analyzer-engine-selection.md)

## Why This Evaluation Exists

ADR-0096 selected but did not admit YARA v4.5.8. The next roadmap step would
freeze an engine-specific build and ruleset. Before doing that, the official
v4.5.8 project metadata disclosed that YARA is now maintenance-focused and
that feature development has shifted to YARA-X.

## Exact Observations

### YARA

- Repository: `VirusTotal/yara`
- Release: `v4.5.8`
- Tag commit: `84b0e3cc0e42f8f8e6b84d19c97ec3ac6ff8aee8`
- Published: `2026-07-28T07:12:15Z`
- Uploaded release assets: `0`
- License: BSD-3-Clause; `COPYING` git blob
  `81b0eed4fe55ab6a33432b140b0f98e61085a5ea`
- Upstream README: maintenance mode; future large features/modules focus on
  YARA-X.
- CLI: compiled rules require `-C`; output is text; process scanning is enabled
  by default in the build and can be disabled with `--disable-proc-scan`.

### YARA-X

- Repository: `VirusTotal/yara-x`
- Release: `v1.20.0`
- Tag commit: `60ad06971467029e77967e59d580cbbe85a1474d`
- Published: `2026-08-24T12:32:25Z`
- License: BSD-3-Clause
- Uploaded release assets: `6`, covering macOS arm64/x86_64, Linux
  arm64/x86_64, Windows x86_64, and a Windows C API package; every asset exposes
  a GitHub-recorded SHA-256 digest.
- Upstream README: Rust rewrite, mature/stable, production-used, and intended
  to replace YARA.
- CLI: compiled rules supported; structured `json` and `ndjson` scan outputs;
  timeout and maximum-match controls; no process scanning.
- Compatibility: C/C++, Python, and Go APIs are not drop-in compatible; most
  rules are intended to remain compatible, with documented differences.

## Decision Effects

The selected YARA-X direction creates near-term contract migration work but aligns the first
analyzer with the active upstream line and offers a narrower machine-readable
CLI boundary. The unselected legacy-YARA alternative would have minimized immediate ADR renaming but requires a
bounded legacy text parser and accepts maintenance-line migration debt.

Neither option removes the requirements for per-target artifact identity,
ruleset ownership/review, SBOM, provenance, signatures, vulnerability/license
review, expiry/revocation, production-admitted IAR-1B confinement, hosted
evidence, or the applicable independent human security review.

ADR-0098 implements the selected replacement contract without admitting or
running YARA-X. It records the official assets as candidates and requires a
separately pinned, rebuilt, reviewed, and Impresari-signed production artifact.

## Non-Claims

No source archive or release asset was downloaded. No binary, ruleset, or
parser was created. No analyzer ran. GitHub-recorded asset digests are metadata,
not an Impresari signature or admission. This evaluation does not claim engine
security, rule compatibility, detection quality, IAR-2, confinement,
production support, or safety.
