# macOS Local-VM Unsigned Synthetic Bundle Assembly Architecture

- Status: Accepted for source-free implementation; distribution remains gated
- Date: 2026-09-01
- PRD: [macOS Local-VM Unsigned Synthetic Bundle Assembly PRD](../product/macos-local-vm-unsigned-bundle-assembly-prd.md)
- Decision: [ADR-0108](../decisions/0108-assemble-a-non-runnable-macos-bundle-before-signing.md)

## Context

ADR-0107 closed the future component-role and cask lifecycle contract. This
checkpoint proves only deterministic filesystem composition. It deliberately
does not substitute real binaries or guest assets because doing so would add
build, release-identity, signing, and execution questions prematurely.

## Flow

```text
ADR-0107 contract + ADR-0091 metadata seal
                  |
                  v
      closed synthetic assembly spec
                  |
          +-------+-------+
          v               v
 private temp run 1  private temp run 2
          |               |
          +-------+-------+
                  v
        exact canonical tree digest
                  |
                  v
          cleanup both roots
                  |
                  v
      source-free receipt only
```

## Tree model

The ADR-0107 `bundle_layout` remains the closed inventory of product roles. The
assembly layer adds only directory parents, required `Contents/Info.plist`, and
a `SYNTHETIC-ONLY.txt` guest marker. Its 13 entries are the complete temporary
tree. Nothing else is permitted.

All regular files have mode `0644`. This intentionally leaves the apparent CLI
and helper destinations non-executable. Directories use `0755` inside a private
`0700` temporary parent on macOS/POSIX. No symlink or special file is permitted.
Windows CI retains the portable tree, determinism, non-symlink, and cleanup
checks, but its Ruby mode bits are not treated as proof of macOS directory
privacy.

The canonical tree digest uses one sorted UTF-8 line per entry:

```text
path<TAB>kind<TAB>mode<TAB>bytes<TAB>sha256-or-none<LF>
```

## Inputs

- exact ADR-0107 package contract;
- exact ADR-0091 guest-release metadata seal;
- generated synthetic `Info.plist` bytes;
- generated role/path-labelled non-executable marker bytes.

No repository source files, compiled executables, provider data, credentials,
or downloaded content enter the tree.

## Failure behavior

Assembly fails before a receipt if an input digest changes, a path escapes the
app root, a destination repeats, an unexpected entry appears, any file becomes
executable, a byte/mode/digest differs, the two runs diverge, or cleanup leaves
either temporary root present. A macOS/POSIX run also fails unless the temporary
parent has exact mode `0700`; Windows performs structural-only CI and makes no
ACL or POSIX-mode claim.

## Sequencing

A later decision may substitute exact unsigned release-candidate binaries and
guest assets only after defining their build, source-revision, license,
vulnerability, and release-identity bindings. Signing, notarization, cask
creation, installation, publication, VM launch, and analyzers remain separate
gates.
