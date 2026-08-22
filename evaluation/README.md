# Local Evaluation Corpus

The v1 manifest defines twelve original, generated synthetic repositories and
their frozen ground truth. Three fixtures are held out (25%). The manifest
records split, language, task rationale, query capability, required evidence,
generator parameters, license, publication class, reviewer, and limitations.

Run the public local subset with:

```sh
cargo run -p context-evaluation --locked --offline
ruby scripts/check-evaluation.rb
```

The harness performs all searches through `context-engine`, compares returned
paths with declared ground truth, independently scores a deterministic native
search baseline and its file-byte context cost, re-expands exact evidence, mutates source to verify stale
rejection, checks bounded packet accounting, checks normalized repeatability,
and fails when the initial PRD thresholds are missed. The frozen result is
`artifacts/evaluation-local.json` and is bound to the manifest SHA-256 digest.

This local subset does not satisfy the separately gated public-repository,
independent-human-review, native platform, or release-candidate experiments.
It makes no comparative claim about LeanCTX, Graft, or another project.

The separate `scale-eval` binary generates 2,000-file and 5,000-file nested
profiles. Five repetitions report cold/warm snapshot and lexical-query
p50/p95/max, cache/source ratios, and explicit partial behavior. The checked
macOS/aarch64 artifact also records peak RSS measured around the complete run.
Run `ruby scripts/check-scale-evaluation.rb`; platform wrappers may enrich new
native artifacts without changing the safe portable runner.
