# macOS XPC feasibility prototype

This directory contains a development-only ADR-0074 IAR-1B feasibility
prototype. It is not installed, published, notarized, or used by ordinary
Impresari Context scans.

The prototype contains one non-UI App Sandbox host, one embedded App Sandbox
XPC service, and a synthetic stand-in for the selected Rust supervisor. It
accepts only fixed synthetic requests and returns bounded source-free receipts.
It exercises native access denials, CPU and address-space-growth limits,
descendant denial, crash/relaunch, exact-target timeout termination, and
synthetic-byte cleanup. It has no network entitlement, repository path,
credential, analyzer, parser, updater, provider, or production signing key.

Passing this prototype does not admit a production macOS analyzer backend or
set `os_confined` to true. Device denial, production profiles, Developer ID
signing/notarization, ADR-0076 cask lifecycle, the complete Tier A corpus, and
multi-host maintenance gates remain mandatory.
