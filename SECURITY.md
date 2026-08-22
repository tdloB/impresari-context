# Security policy

## Reporting a vulnerability

Use GitHub's **Report a vulnerability** button on this repository's Security
page. This sends the report privately to the maintainer without requiring a
public email address. The maintainer must enable private vulnerability reporting
when the repository becomes public and keep GitHub security notifications
monitored.

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

See [the threat model](docs/security/threat-model.md) for the normative design
baseline.
