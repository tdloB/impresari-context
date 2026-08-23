# Autonomous Roadmap Execution

The founder directs Codex to continue all accepted Impresari Context roadmap work autonomously.

- Treat design, implementation, local quality gates, pull-request creation, CI monitoring, failure remediation, merge, and transition to the next accepted milestone as one continuous workflow.
- Do not stop, ask for confirmation, or issue a final handoff merely because an ordinary milestone completes.
- Stop only for a required manual user action, an unavoidable platform approval, an unresolved technical failure after safe in-scope alternatives, or completion of all accepted roadmap work.
- Do not infer approval for a materially new product scope. If no accepted milestone remains, complete all non-dependent work and state the precise product decision needed to continue.
- Keep the user informed with concise progress updates while work is active.

These instructions supplement, and never override, higher-priority system, developer, safety, or platform requirements.

# AI App Builder OS Delivery Guardrail

Build Impresari Context using the private AI App Builder OS as the governing
delivery guardrail. Its source of truth is
`/Users/aaronboldt/Documents/AI Build Agents/ai-app-builder-os`.

- Start each material roadmap increment through the OS Orchestrator process.
  Use the existing-application and Architecture Evolution Review guidance when
  a change affects architecture, trust boundaries, public contracts, quality
  thresholds, dependencies, or another hard-to-reverse decision.
- Apply the Solution Architecture standards to component boundaries, authority
  separation, dependencies, sequencing, non-goals, tradeoffs, and acceptance
  evidence before implementation hardens.
- Apply the applicable language-best-practices specialist standards (including
  the Rust specialist for this workspace) to production code and dependencies.
- Apply the Testing and Quality Engineering standards to a risk-based test
  strategy, negative and boundary cases, regression coverage, repeatable local
  validation, and required hosted evidence.
- Apply the Security, Privacy, and Trust standards to threat modeling, input
  validation, privilege/authority boundaries, dependency risk, secret safety,
  failure handling, and security-sensitive test cases. Escalate unresolved
  material security risk rather than accepting it implicitly.
- Apply the Repository Delivery standards to every remote mutation: exact
  repository and base SHA, scoped diff review, validation evidence, protected
  checks, and independent remote-SHA verification.
- Keep private OS process records and specialist artifacts in the OS private
  area. Do not copy private OS prompts, agent definitions, credentials, or
  internal reports into this public repository.

# Dogfood Impresari Context for Repository Context

Use Impresari Context itself as the preferred local context-engineering layer
for this repository when its current CLI or MCP surface can produce a bounded,
snapshot-bound packet for the task. Treat that packet as evidence support under
the OS Orchestrator, not as routing or approval authority. Validate important
claims against exact source and fall back to direct, bounded inspection when a
packet is unavailable, stale, incomplete, or less efficient for the question.
