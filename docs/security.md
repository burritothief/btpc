# Repository security operations

Maintainers own dependency, CodeQL, Scorecard, secret-scanning, and private
vulnerability alerts. The maintainer handling an alert records severity, affected
versions, an owner, remediation plan, and disclosure decision in the corresponding
private report or tracking issue.

## GitHub settings checklist

- Enable Dependabot alerts and security updates. Dependabot version updates remain
  review-only; do not enable automatic merging.
- Enable private vulnerability reporting and publish `SECURITY.md` as the reporting
  policy.
- Enable secret scanning and push protection, including alerts for contributors
  where the repository plan supports them.
- Protect `main` with the required checks in `CONTRIBUTING.md`, required approving
  review, stale-review dismissal, conversation resolution, and current-branch
  enforcement. Disable force pushes and deletion.
- Restrict workflow permissions to read-only by default and allow GitHub Actions to
  create pull requests only if a reviewed workflow later requires it.
- Review CodeQL and Scorecard SARIF in code scanning. Assign every actionable alert
  before dismissal and document false-positive rationale.

## Dependency exceptions

Dependency Review rejects high or critical vulnerabilities and dependencies outside
the license allowlist shared with `deny.toml`. An exception requires maintainer
approval in a dedicated issue containing the advisory/license, affected dependency,
rationale, compensating controls, owner, remediation target, and an expiration date
no more than 90 days away. The configuration change must link to that issue. Remove
expired exceptions immediately; renewal requires a new review.

## Coverage policy

Rust and Python coverage are informational during pre-1.0 development and are not
merged into a misleading cross-language percentage. Artifacts retain the native
LCOV and Coverage.py XML formats. A future threshold must be based on an accepted
baseline and prevent regression in protocol-critical modules rather than reward
low-value line coverage.
