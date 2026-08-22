# Bounded Git Metadata

Snapshot status reports optional Git metadata without invoking Git:

- `.git` must be a real directory directly beneath the authorized root;
- `.git` links and `gitdir:` files are never followed;
- `HEAD`, a capability-relative loose ref, or `packed-refs` is read under fixed
  byte ceilings with the same component/link defenses as source evidence;
- only 40- or 64-character hexadecimal object identities are accepted;
- malformed, unborn, unsupported, or inaccessible layouts omit the revision;
- working-tree state is `unknown` for detected Git layouts because proving
  clean/dirty would require broader Git index/object semantics outside Slice A;
- non-Git workspaces report `not_applicable`.

Repository metadata is informative provenance. Exact source evidence remains
bound to the content-derived workspace snapshot and never trusts Git metadata
as its sole authority.
