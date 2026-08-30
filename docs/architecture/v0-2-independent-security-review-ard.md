# v0.2 Independent Security Review ARD

## Components

- `release-review/v0.2.0-independent-review-scope.json` is the immutable review
  handoff scope.
- `independent-security-review.schema.json` defines pending and future recorded
  scope shapes plus the source-free receipt.
- `independent-security-review-readiness.rb` evaluates already-observed metadata
  without reading source, starting a process, contacting a reviewer, or changing
  release state.
- The reviewer brief describes the human work product and required evidence.

## Binding model

The evaluator pins the tracked scope SHA-256. The scope pins the version and
product-source commit. A caller cannot substitute a permissive scope or assert
that another version or commit was reviewed. A future report-admission change
must update the tracked scope, add the exact report identity and attributable
review metadata, and update the evaluator's pinned identity under normal review.

The report itself may be retained as a repository document, a public immutable
artifact, or a private report whose digest and bounded public summary are
recorded. Sensitive exploit details need not be published. The attributable
reviewer reference, exact reviewed commit, finding counts, dispositions, and
report SHA-256 are mandatory.

## State machine

`manual_review_required` is the only positive state reachable from the current
scope and still means the gate is unsatisfied. `changed`, `missing_evidence`,
`invalid_review`, and `unsupported` fail closed. A later `review_admitted` state
may satisfy only `review_gate_satisfied`; release readiness, publication,
production support, analyzer execution, risk acceptance, and every evaluator
authority remain false.

## Release ancestry

The reviewed product commit is the behavioral baseline. If any crate, runtime
script, workflow affecting packaged bytes, schema governing runtime behavior,
or security policy changes after review, the scope must be refreshed and the
reviewer must assess the delta or repeat the review. The final tag may descend
from the reviewed commit only through transparently enumerated review evidence,
version, changelog, and release-record changes that do not alter product
behavior. The pre-publication gate must verify that path list explicitly.
