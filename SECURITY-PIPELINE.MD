# Security Pipeline Implementation

This document describes the security pipeline implemented for the Snipping Bot project, based on the OWASP guidelines in [10Owasps.md](file:///d:/auto-sync-daily-from-winC/snipping-bot/10Owasps.md).

## Overview

The security pipeline implements checks for:

1. **OWASP Top 10 (Web, 2021)**
2. **OWASP API Security Top 10 (2023)**
3. **OWASP Mobile Top 10 (2024)**
4. **OWASP Top 10 Proactive Controls (2024)**

## CI/CD Pipeline Stages

### 1. Pre-commit / Local Checks

- `cargo fmt --check` - Code formatting
- `cargo clippy` - Code linting
- `gitleaks` - Secrets scanning

### 2. Supply Chain Security

- `cargo audit` - Vulnerability scanning for dependencies
- `cargo deny` - License compliance and dependency policies
- SBOM generation with `syft`

### 3. Static Analysis

- `cargo udeps` - Detection of unused dependencies
- `cargo machete` - Detection of unused Cargo.toml entries
- `semgrep` - Pattern-based static analysis

### 4. Infrastructure Security

- IaC scanning with `checkov` for Kubernetes/Helm configurations
- Container scanning with `Trivy`
- Dockerfile best practices

### 5. API Security

- API schema validation
- Contract testing with `schemathesis`

### 6. Dynamic Analysis

- OWASP ZAP baseline scanning

## GitHub Actions Workflows

### Main CI Pipeline (`ci.yml`)

Enhanced with security checks:

- Security audit with `cargo-audit`
- License compliance with `cargo-deny`
- Secrets scanning with `gitleaks`
- Unused dependencies check with `cargo-udeps` (full scan only)

### Dedicated Security Pipeline (`security-ci.yml`)

Comprehensive security pipeline that runs:

- All checks from main CI pipeline
- Additional static analysis with `cargo-machete` and `semgrep`
- Infrastructure as Code scanning with `checkov`
- Container scanning with `Trivy`
- API contract testing with `schemathesis`
- Dynamic analysis with OWASP ZAP
- Scheduled daily full scans

## Local Development Scripts

### Security Checks Script (`scripts/run-security-checks.ps1`)

Developers can run security checks locally:

```powershell
# Quick security checks (default)
.\scripts\run-security-checks.ps1

# Full security checks
.\scripts\run-security-checks.ps1 -Level full
```

### Compliance Tests Script (`scripts/run_compliance_tests.ps1`)

Already existing script that tests security-related crates:

- `sniper-security`
- `sniper-telemetry`
- `sniper-keys`
- `sniper-risk`
- `sniper-policy`

## Security Tools

### Dependency and Vulnerability Scanning

- **cargo-audit**: Scans for known vulnerabilities in dependencies
- **cargo-deny**: Enforces license compliance and dependency policies

### Secrets Detection

- **gitleaks**: Scans for hardcoded secrets in the codebase

### Static Analysis

- **cargo-udeps**: Detects unused dependencies
- **cargo-machete**: Detects unused entries in Cargo.toml
- **semgrep**: Pattern-based static analysis

### Infrastructure Security

- **checkov**: Infrastructure as Code scanning
- **Trivy**: Container image and filesystem scanning

### API Security

- **schemathesis**: Automated API testing based on OpenAPI specifications

### Dynamic Analysis

- **OWASP ZAP**: Dynamic Application Security Testing

## Configuration Files

### `deny.toml`

Configures license compliance and dependency policies:

- Allowed licenses: MIT, Apache-2.0, BSD-3-Clause, etc.
- Ignored advisories (with justification)
- Crate exceptions

### `.github/workflows/security-ci.yml`

Full security pipeline configuration with:

- Scheduled daily scans
- Quick vs. full scan options
- Comprehensive security checks
- Security reporting

## Running Security Checks

### In CI/CD

Security checks automatically run on:

- Every push to `main` branch
- Every pull request to `main` branch
- Daily scheduled runs (full scans)

### Locally

Developers can run security checks locally:

```powershell
# Quick security checks
.\scripts\run-security-checks.ps1

# Full security checks
.\scripts\run-security-checks.ps1 -Level full
```

### Manual Cargo Commands

Direct cargo commands for specific checks:

```bash
# Security audit
cargo audit --deny warnings

# License compliance
cargo deny check bans sources licenses

# Unused dependencies
cargo udeps --all-targets

# Unused Cargo.toml entries
cargo machete

# Code formatting
cargo fmt --all -- --check

# Code linting
cargo clippy --all-targets --all-features -- -D warnings
```

## Security Gate Thresholds

The pipeline enforces these security thresholds:

- **cargo audit**: No CRITICAL or HIGH severity vulnerabilities
- **cargo deny**: No banned licenses or sources
- **gitleaks**: No secrets detected
- **cargo udeps**: No unused dependencies
- **cargo machete**: No unused Cargo.toml entries
- **clippy**: No warnings (treated as errors)
- **Trivy**: No CRITICAL vulnerabilities in containers

## Future Enhancements

Planned security improvements:

1. Integration with OPA/Cedar for policy-as-code
2. Enhanced SSRF protection testing
3. Rate limiting and resource consumption tests
4. Business flow abuse detection
5. Provenance attestation for releases
6. Admission policy enforcement for deployments

## Troubleshooting

### Common Issues

1. **cargo-audit fails**: Update dependencies to fix vulnerabilities
2. **cargo-deny fails**: Check license compliance or add exceptions
3. **gitleaks detects secrets**: Remove secrets and rotate them
4. **cargo-udeps finds unused deps**: Remove unused dependencies
5. **Clippy warnings**: Fix code issues or adjust clippy configuration

### False Positives

For false positives in security scans:

1. **cargo-audit**: Add to `ignore` list in `deny.toml` with justification
2. **gitleaks**: Add to `.gitleaksignore` or adjust rules
3. **cargo-deny**: Add exceptions to `deny.toml` with justification

## References

- [OWASP Top 10](https://owasp.org/Top10/)
- [OWASP API Security Top 10](https://owasp.org/API-Security/)
- [OWASP Mobile Top 10](https://owasp.org/www-project-mobile-top-10/)
- [OWASP Top 10 Proactive Controls](https://top10proactive.owasp.org/)
- [OWASP Application Security Verification Standard (ASVS)](https://owasp.org/www-project-application-security-verification-standard/)