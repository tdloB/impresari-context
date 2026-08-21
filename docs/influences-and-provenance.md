# Influences and implementation provenance

## 1. Independent-project statement

This is an independent project informed by publicly documented capabilities
and observed behavior in other context-engineering and code-intelligence
systems. It is not a fork, merger, or official successor.

No LeanCTX or Graft source code is to be copied, translated, or adapted into
the implementation unless the project deliberately changes its provenance and
license-compliance model in a documented decision before that code is added.

## 2. Acknowledgments

The architecture was informed in part by:

- [LeanCTX](https://github.com/yvgude/lean-ctx), created by Yves Gugger and
  contributors. Its public work demonstrates compressed retrieval, recoverable
  context, session state, context budgets, multi-agent handoffs, extensibility,
  and security controls for AI context infrastructure.
- [Graft](https://github.com/nanonets/graft), developed by the Context Graph
  Engine contributors and published by Nanonets. Its public work demonstrates
  a compact deterministic structural-graph interface, source-linked concepts,
  repository maps, call tracing, and freshness-aware queries.

The new project is not affiliated with or endorsed by either upstream project.
Their names are used only to identify architectural influences.

## 3. Adopted architectural lessons

The project adopts general capabilities, not upstream expression or internal
structure:

| General lesson | Independent direction |
|---|---|
| Context can be compressed | Compression must retain exact recovery references |
| Context must remain fresh | Packets bind to content-addressed snapshots |
| Code structure benefits retrieval | One deterministic graph is canonical |
| Agents need bounded tool surfaces | One small capability vocabulary spans transports |
| Sessions and handoffs reduce rediscovery | Handoffs carry immutable packets, not routing authority |
| Extensions broaden usefulness | Capability manifests and normalization constrain extensions |
| Derived summaries can help | Exact evidence remains authoritative |
| Tooling should be measurable | Evaluation and local observability are core components |

## 4. Deliberately rejected patterns

The initial design rejects:

- overlapping graph engines presented to the same client;
- global shell or editor mutation during setup;
- unpinned latest-version execution;
- automatic self-update inside a work session;
- mandatory provider proxying;
- arbitrary command execution as a context feature;
- LLM-required canonical indexing;
- unreviewed durable-memory promotion;
- autonomous agent routing inside the context engine;
- repository text being interpreted as system instructions;
- security findings that cannot be expanded to exact source.

## 5. Implementation rules

Contributors must:

1. Implement against this project's specifications and tests.
2. Use original names, schemas, control flow, error models, documentation, and
   test fixtures.
3. Avoid copying or mechanically translating upstream code, comments, prompts,
   examples, documentation, or visual assets.
4. Record externally derived protocol constraints or compatibility behavior in
   a design decision or compatibility note.
5. Identify third-party libraries normally through the dependency manifest and
   automated license inventory.
6. Stop implementation and open a provenance decision before importing source
   from LeanCTX, Graft, or another project.
7. Never describe this work as a formal legal clean-room implementation unless
   a qualified process has been established and reviewed separately.

Public source has already been inspected during architectural research, so the
accurate description is `independent implementation`, not `clean room`.

## 6. Future source-reuse gate

If source reuse is proposed later, the change must include before merge:

- exact upstream repository and immutable revision;
- original and destination paths;
- applicable license and copyright notices;
- modification description;
- compatibility analysis with the project license;
- required `NOTICE` or third-party-license changes;
- explicit maintainer approval within this project;
- an update replacing any no-upstream-code statement that would become false.

Until that gate is completed, the repository must continue to state that no
LeanCTX or Graft code is incorporated.

## 7. Maintainer outreach

Outreach to the LeanCTX and Graft maintainers is encouraged after the
architecture and acknowledgment language are reviewable and before the public
launch announcement. The purpose is transparency, preferred attribution,
technical feedback, and possible collaboration. It is not presented as an
endorsement request or as a substitute for license compliance.
