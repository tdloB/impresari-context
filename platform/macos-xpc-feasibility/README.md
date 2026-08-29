# macOS XPC feasibility prototype

This directory contains a development-only ADR-0074 IAR-1B feasibility
prototype. It is not installed, published, notarized, or used by ordinary
Impresari Context scans.

The prototype contains one non-UI App Sandbox host and one embedded App
Sandbox XPC service. It accepts only a fixed synthetic request and returns a
bounded source-free receipt. It has no network entitlement, repository path,
credential, analyzer, parser, updater, provider, or production signing key.

Passing this prototype does not admit a production macOS analyzer backend or
set `os_confined` to true. The complete native escape, resource, descendant,
cleanup, packaging, and maintenance gates remain mandatory.
