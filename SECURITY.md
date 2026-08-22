# Security policy

## Reporting a vulnerability

Use GitHub's **Report a vulnerability** button on the
[private vulnerability reporting page](https://github.com/tdloB/impresari-context/security/advisories/new).
This sends the report privately to the maintainer without requiring a public
email address. Private vulnerability reporting is enabled, and the maintainer
monitors GitHub security notifications.

Do not open public issues containing vulnerabilities, secrets, private source
code, exploit details, or personal data. If the private reporting button is not
available, open a public issue containing no sensitive details and ask the
maintainer (`@tdloB`) to establish a private channel.

## Supported versions

Until `v0.1.0` is published, no versions are supported. After publication, the
latest `0.1.x` release will receive security fixes; older prerelease builds are
unsupported.

## Handling

Maintainers will record severity, affected versions, remediation, advisory,
release, and coordinated-disclosure decisions while minimizing reporter and
private-source data. Critical authorization or workspace-isolation failures are
release blockers and cannot be silently waived.

The maintainer aims to acknowledge a report within 3 business days and provide
an initial assessment within 7 business days. Remediation and coordinated
disclosure timing depends on severity and complexity; the reporter will receive
status updates at least every 14 days while a confirmed vulnerability remains
open. Please do not disclose a vulnerability publicly before a coordinated
advisory or an agreed disclosure date.

See [the threat model](docs/security/threat-model.md) for the normative design
baseline.
