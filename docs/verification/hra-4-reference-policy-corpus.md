# HRA-4 Reference Admission Policy Corpus

- Scope: pure deterministic evaluation of exact immutable HRA records.
- Runtime authority: none.
- Contract version: `1.0.0`.

## Input closure

The evaluator accepts only a repository security assessment, its exact analyzer
coverage ledger, the exact finding set named by the assessment, and a closed
repository admission policy. It recomputes the policy digest, validates the
assessment and coverage identities through the evidence crate, and requires the
finding identity set and workspace snapshot to match exactly.

There is deliberately no filesystem path, command, environment, network,
credential, model, exception, approval, or user-authority parameter. Unknown
policy fields are rejected during deserialization.

## Truth table

| Strongest condition | Required outcome |
| --- | --- |
| Any matching `block` rule | `blocked` |
| Otherwise, any matching `manual_review` rule | `manual_review_required` |
| Otherwise, any matching `require_analysis` rule | `analysis_incomplete` |
| Otherwise, incomplete assessment or mandatory coverage | `analysis_incomplete` |
| Otherwise, exactly one matching eligibility profile | `isolated_execution_eligible` |
| Otherwise, including no match or conflicting profiles | `manual_review_required` |

Every matched rule is retained in canonical priority/rule-ID order even when a
stricter rule determines the outcome. Missing mandatory analyzer capabilities
are separately returned in sorted unique order.

## Monotonic restriction

The effect rank is fixed as `blocked` > `manual_review_required` >
`analysis_incomplete` > `isolated_execution_eligible`. Adding a matching rule or
finding cannot suppress a stricter existing effect. Adding missing coverage
cannot make a decision eligible. A blocking match removes any quarantine
profile from the output.

## Authority statement

`isolated_execution_eligible` is only a deterministic classification naming one
future disposable-quarantine profile. It does not provision that profile,
approve risk, authorize ordinary-host execution, or prove the repository safe.
All decision authority and safety fields are constant false.
