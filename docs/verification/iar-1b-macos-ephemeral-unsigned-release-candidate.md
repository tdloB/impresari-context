# IAR-1B macOS Ephemeral Unsigned Release Candidate Evidence

- Decision: [ADR-0114](../decisions/0114-build-assemble-verify-and-delete-the-macos-unsigned-candidate.md)
- Scope: macOS arm64, synthetic guest, unsigned metadata-only candidate record
- Date: 2026-09-01

## Observed

- exact source archive and host/toolchain identity;
- four exact arm64 product binaries, two exact synthetic guest files,
  deterministic `Info.plist`, and exact guest metadata seal;
- one closed eight-file app tree under one private root;
- exact `0755` product and `0644` metadata/guest modes;
- ADR-0109 compound identity
  `sha256:8d3da788a95c6cf638537218722e5fe32629710a10a3b25c0ac282280ed5720e`;
- ADR-0113 material identity
  `sha256:39ae0afbb77eff80ff5308cc4fe811b7cc266b42d02b4457aa5295310908b11e`;
- complete deletion before metadata acceptance.

## Not admitted

No artifact was retained or executed. No Apple identity, Developer ID
signature, notarization, archive, cask, install, publication, VM launch,
analyzer execution, production support, or macOS IAR-1B admission is claimed.
