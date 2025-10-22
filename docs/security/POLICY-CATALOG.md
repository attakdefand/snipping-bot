# Security Policy Catalog

This document serves as the single source of truth for security policies mapped to controls and standards as defined in the security layers checklist.

## Table of Contents

1. [Governance & Policy](#1-governance--policy)
2. [Risk & Threat Modeling](#2-risk--threat-modeling)
3. [Secure SDLC & Supply Chain](#3-secure-sdlc--supply-chain)
4. [Identity & Access (IAM)](#4-identity--access-iam)
5. [Secrets Management](#5-secrets-management)
6. [Key & Cryptography](#6-key--cryptography)
7. [Network Segmentation & Transport](#7-network-segmentation--transport)
8. [Perimeter & API Gateway](#8-perimeter--api-gateway)
9. [Host/Endpoint Hardening](#9-hostendpoint-hardening)
10. [Containers & Orchestration](#10-containers--orchestration)
11. [Cloud/IaaS Security](#11-cloudiaas-security)
12. [Data Security](#12-data-security)
13. [Application Security](#13-application-security)
14. [Protocol/API Security](#14-protocolapi-security)
15. [Messaging & Event Security](#15-messaging--event-security)
16. [Database Security](#16-database-security)
17. [Wallet/Custody & Key Ops (Web3)](#17-walletcustody--key-ops-web3)
18. [Oracle & Market Data Integrity (Web3)](#18-oracle--market-data-integrity-web3)
19. [Privacy & Compliance](#19-privacy--compliance)
20. [Observability & Telemetry Security](#20-observability--telemetry-security)
21. [Detection & Response](#21-detection--response)
22. [Resilience, Availability & Chaos](#22-resilience-availability--chaos)

## 1. Governance & Policy

### Security Policy Framework
- **Control**: Security Policy Catalog
- **Description**: Single source of truth for policies mapped to controls and standards
- **Component**: Repo: /docs/security, ADRs
- **Artifact**: docs/security/POLICY-CATALOG.md
- **Test Category**: Policy Linting, Documentation Tests
- **Metric/KPI**: % policy coverage; policy->control mapping completeness
- **Evidence**: Signed policy documents, change history

### Exception Management
- **Control**: Risk-accepted Exceptions
- **Description**: Formal process for temporary deviations with expiry and owner
- **Component**: GRC register; Git CODEOWNERS
- **Artifact**: docs/security/EXCEPTIONS.md
- **Test Category**: Exception Expiry Tests
- **Metric/KPI**: # open exceptions; avg time to close
- **Evidence**: Approved exception records with sign-off

### Audit & Assurance
- **Control**: Internal/External Audits
- **Description**: Track findings, owners, due dates, remediation status
- **Component**: Issue tracker labels: security-audit
- **Artifact**: docs/security/AUDIT-FINDINGS.md
- **Test Category**: Evidence Collection, Control Verification
- **Metric/KPI**: % findings closed on time
- **Evidence**: Audit reports, remediation PRs

### Standards Mapping
- **Control**: Control->Standard Traceability
- **Description**: Map controls to ISO/NIST/OWASP for compliance reuse
- **Component**: Matrix in repo
- **Artifact**: docs/security/STANDARDS-MAP.csv
- **Test Category**: Matrix Consistency Tests
- **Metric/KPI**: % controls mapped; gaps
- **Evidence**: Traceability matrix CSV

## 2. Risk & Threat Modeling

### Methodologies
- **Control**: STRIDE / LINDDUN / Attack Trees
- **Description**: Apply per service and per data flow
- **Component**: docs/diagrams; threat-model.json
- **Artifact**: docs/security/THREAT-MDL.md
- **Test Category**: Model Linting, DFD Consistency
- **Metric/KPI**: models per service; refreshed in last 90d
- **Evidence**: DFDs, threat register, mitigations list

### Abuse Cases
- **Control**: Misuse/Abuse Scenarios
- **Description**: Document adversarial flows and required controls
- **Component**: Abuse-case tests in QA
- **Artifact**: tests/abuse/
- **Test Category**: Abuse-Case Tests
- **Metric/KPI**: # abuse cases covered
- **Evidence**: Test artifacts, screenshots, logs

### Risk Register
- **Control**: Risk Scoring & Owners
- **Description**: Qual/quant scoring with treatment plan
- **Component**: risk.yaml
- **Artifact**: docs/security/RISK-REGISTER.yaml
- **Test Category**: Drift Checks, SLA Alerts
- **Metric/KPI**: % risks with owners; MTTR per risk
- **Evidence**: Signed risk register snapshots

## 3. Secure SDLC & Supply Chain

### Code Scanning
- **Control**: SAST/Secret Scan
- **Description**: Rust Clippy/deny, secret scanners on PR
- **Component**: cargo-deny, trufflehog
- **Artifact**: .github/workflows/security.yml
- **Test Category**: SAST, Secret Leak Tests
- **Metric/KPI**: % PRs scanned; # leaks blocked
- **Evidence**: CI logs, SARIF reports

### Dependency Health
- **Control**: SCA/SBOM
- **Description**: Lockfile vetting, SBOM generation and verification
- **Component**: cargo auditable, syft
- **Artifact**: sbom/syft.spdx.json
- **Test Category**: SCA, SBOM Diff Tests
- **Metric/KPI**: Known vuln exposure; SBOM freshness
- **Evidence**: SPDX files, attestations

### Build Integrity
- **Control**: Sigstore/Cosign & Reproducible Builds
- **Description**: Sign images/artifacts; verify in deploy
- **Component**: cosign, GitHub OIDC
- **Artifact**: attestations/cosign.json
- **Test Category**: Signature Verification Tests
- **Metric/KPI**: % artifacts signed & verified
- **Evidence**: Cosign attestations

### PR Gates
- **Control**: Security PR Checks
- **Description**: Required checks block merge (linters, tests, policy)
- **Component**: Branch protection rules
- **Artifact**: CODEOWNERS, .github/
- **Test Category**: Policy-as-Code Tests
- **Metric/KPI**: Block rate for insecure PRs
- **Evidence**: Merge logs, check run outputs

## 4. Identity & Access (IAM)

### AuthN
- **Control**: OAuth2/OIDC, MFA
- **Description**: Central IdP, MFA enforced for admin
- **Component**: Keycloak/Okta; Rust oidc crates
- **Artifact**: config/idp.json
- **Test Category**: AuthN Tests
- **Metric/KPI**: % admins with MFA; login success/deny rates
- **Evidence**: IdP audit logs, MFA proof

### AuthZ
- **Control**: RBAC/ABAC, Least Privilege
- **Description**: Scoped tokens; service/service roles
- **Component**: OPA/Cedar policies
- **Artifact**: policy/authz.rego
- **Test Category**: Authorization Tests
- **Metric/KPI**: Denied-by-default coverage
- **Evidence**: Policy bundles, decision logs

### JIT Access
- **Control**: Just-in-Time & Break-glass
- **Description**: Time-bound elevation with approval & logging
- **Component**: Access broker tool
- **Artifact**: runbooks/JIT.md
- **Test Category**: Access Flow Tests
- **Metric/KPI**: # JIT grants; expiry enforcement
- **Evidence**: Grant tickets, session logs

### Key Rotation
- **Control**: Credential & Token Rotation
- **Description**: Short TTL tokens; periodic rotation jobs
- **Component**: cronjobs in K8s
- **Artifact**: k8s/cron/rotate.yaml
- **Test Category**: Rotation Tests
- **Metric/KPI**: Age of oldest secret; rotation success
- **Evidence**: Rotation logs, secret versions

## 5. Secrets Management

### Storage
- **Control**: Vault/SOPS
- **Description**: Central secret store, envelopes for CI/CD
- **Component**: HashiCorp Vault; sops + age
- **Artifact**: infra/vault/
- **Test Category**: Secret Access Tests
- **Metric/KPI**: # direct env secrets (should be 0)
- **Evidence**: Vault policies, access logs

### Separation
- **Control**: Env Separation & Scoping
- **Description**: Dev/Stage/Prod isolation and namespace scoping
- **Component**: K8s namespaces/tenants
- **Artifact**: k8s/namespaces.yaml
- **Test Category**: Env Boundary Tests
- **Metric/KPI**: Cross-env secret access attempts
- **Evidence**: Namespace manifests

### Scanning
- **Control**: Secret Scanning in Repos & Images
- **Description**: Block commits/images with secrets
- **Component**: gitleaks; trufflehog; crane
- **Artifact**: .gitleaks.toml
- **Test Category**: Leak Tests
- **Metric/KPI**: Leak MTTR; # blocked leaks
- **Evidence**: Scanner reports

### Rotation
- **Control**: Automated Rotation
- **Description**: Rotate DB/API keys with zero-downtime
- **Component**: Sidecars/jobs
- **Artifact**: jobs/rotate-*.yaml
- **Test Category**: Rotation Tests
- **Metric/KPI**: Rotation success; outage minutes (should 0)
- **Evidence**: Rotation run logs

## 6. Key & Cryptography

### KMS/HSM
- **Control**: Centralized Key Management
- **Description**: Envelope encryption, HSM for root keys
- **Component**: AWS KMS/CloudHSM or on-prem
- **Artifact**: kms/policies.json
- **Test Category**: Crypto Tests
- **Metric/KPI**: Key usage logs audited; separation of duties
- **Evidence**: KMS key policies, CMKs list

### TLS Policy
- **Control**: Cipher Suites & TLS
- **Description**: Min TLS1.2+, strong ciphers; cert rotation
- **Component**: Linkerd/Ingress TLS
- **Artifact**: k8s/ingress-tls.yaml
- **Test Category**: TLS Tests
- **Metric/KPI**: % services mTLS; cert age
- **Evidence**: TLS scan outputs

### Deterministic/Non-det Crypto
- **Control**: Crypto Rules
- **Description**: Rules for when to use deterministic vs randomized modes
- **Component**: Crypto guidelines doc
- **Artifact**: docs/security/CRYPTO-RULES.md
- **Test Category**: Unit/Property Tests
- **Metric/KPI**: Violations in code scan
- **Evidence**: Guideline doc, code refs

### Key Ops
- **Control**: Split Knowledge, Dual Control
- **Description**: 2-person rule for key export/use
- **Component**: Custody procedures
- **Artifact**: runbooks/KEY-OPS.md
- **Test Category**: Procedure Drills
- **Metric/KPI**: # violations; drill results
- **Evidence**: Drill artefacts, approvals

## 7. Network Segmentation & Transport

### Zero-Trust
- **Control**: Default-Deny East/West
- **Description**: Service identity + policy to allow
- **Component**: Linkerd/NetworkPolicies
- **Artifact**: k8s/networkpolicies.yaml
- **Test Category**: Policy Tests
- **Metric/KPI**: % flows allowed only by policy
- **Evidence**: Policy manifests, mesh config

### mTLS
- **Control**: Service-to-Service AuthN
- **Description**: SPIFEE/SPIRE IDs; cert rotation
- **Component**: Service mesh
- **Artifact**: mesh/config.yaml
- **Test Category**: Handshake Tests
- **Metric/KPI**: mTLS success rate; cert expiries
- **Evidence**: Mesh metrics, cert bundles

### Egress Control
- **Control**: Outbound Allow-List
- **Description**: Block exfil via strict egress
- **Component**: Egress gateways
- **Artifact**: k8s/egress.yaml
- **Test Category**: Egress Tests
- **Metric/KPI**: # unexpected egress blocks
- **Evidence**: Gateway logs

### Ingress Control
- **Control**: North/South Filtering
- **Description**: IP allow-lists, geo-fencing where needed
- **Component**: WAF/Ingress
- **Artifact**: k8s/ingress-waf.yaml
- **Test Category**: Ingress Tests
- **Metric/KPI**: Attack blocked rate
- **Evidence**: WAF rule sets, logs

## 8. Perimeter & API Gateway

### WAF
- **Control**: OWASP Top 10 Filters
- **Description**: Block common web attacks
- **Component**: nginx/modsec/Cloud WAF
- **Artifact**: waf/rules.conf
- **Test Category**: DAST, Probe Tests
- **Metric/KPI**: WAF block/allow precision
- **Evidence**: WAF logs, configs

### Rate Limit
- **Control**: Global & Per-Token Limits
- **Description**: Protect against DoS & abuse
- **Component**: Envoy/Linkerd + Redis
- **Artifact**: gateway/ratelimit.yaml
- **Test Category**: Load/Abuse Tests
- **Metric/KPI**: 429 rate under attack
- **Evidence**: Rate-limit policies

### Schema Validation
- **Control**: Strict Request/Response Schemas
- **Description**: OpenAPI/GraphQL SDL enforcement
- **Component**: Spectral, graphql-shield
- **Artifact**: api/schema/
- **Test Category**: Contract Tests
- **Metric/KPI**: Schema drift incidents
- **Evidence**: Schema snapshots, diffs

### Bot Defense
- **Control**: Bot/Automation Controls
- **Description**: Challenge/behavioral checks on public endpoints
- **Component**: Bot mgmt service
- **Artifact**: gateway/bot.yaml
- **Test Category**: Bot Tests
- **Metric/KPI**: Bot traffic ratio
- **Evidence**: Bot rules, challenge stats

## 9. Host/Endpoint Hardening

### Baselines
- **Control**: CIS Baselines
- **Description**: Harden OS images & configs
- **Component**: Ansible/Packer images
- **Artifact**: infra/images/
- **Test Category**: Config Compliance Tests
- **Metric/KPI**: % hosts compliant
- **Evidence**: CIS scan outputs

### Exploit Mitigations
- **Control**: ASLR/PIE, NX, CFI
- **Description**: Compiler/OS flags enabled
- **Component**: Rust + kernel flags
- **Artifact**: build/flags.toml
- **Test Category**: Binary Checks
- **Metric/KPI**: % binaries PIE/NX
- **Evidence**: objdump results, build logs

### EDR
- **Control**: Endpoint Detection & Response
- **Description**: Agents or kernel sensors
- **Component**: EDR vendor / Falco
- **Artifact**: edr/config.yaml
- **Test Category**: Detection Tests
- **Metric/KPI**: MTTD endpoint threats
- **Evidence**: Alerts, detections

### SSH/Access
- **Control**: SSH Hardening & PAM
- **Description**: No password logins; key-only; session logs
- **Component**: sshd_config; Teleport
- **Artifact**: infra/ssh/sshd_config
- **Test Category**: Access Tests
- **Metric/KPI**: # direct root logins (0)
- **Evidence**: Auth logs

## 10. Containers & Orchestration

### Images
- **Control**: Minimal, Signed, Non-root
- **Description**: Distroless, UID drop, RO FS
- **Component**: Dockerfiles; cosign
- **Artifact**: docker/
- **Test Category**: Image Checks
- **Metric/KPI**: % signed; % non-root
- **Evidence**: Image scan, attestations

### Policies
- **Control**: PSP/OPA Gatekeeper
- **Description**: Deny privileged, hostPath, CAP_SYS_ADMIN
- **Component**: Gatekeeper constraints
- **Artifact**: k8s/policy/
- **Test Category**: Policy Tests
- **Metric/KPI**: # policy violations
- **Evidence**: Constraint templates, audit logs

### Runtime
- **Control**: seccomp/AppArmor
- **Description**: Syscall allowlists
- **Component**: Profiles per service
- **Artifact**: k8s/seccomp/
- **Test Category**: Runtime Tests
- **Metric/KPI**: Syscall block events
- **Evidence**: Auditd/Falco logs

### Scheduling
- **Control**: Node/Pod Security & Quotas
- **Description**: Taints/tolerations, resource limits
- **Component**: k8s/limits.yaml
- **Artifact**: k8s/limits.yaml
- **Test Category**: Chaos/Capacity Tests
- **Metric/KPI**: % pods with limits
- **Evidence**: K8s API evidence

## 11. Cloud/IaaS Security

### Account Guardrails
- **Control**: Org SCP/Projects
- **Description**: Deny dangerous services by default
- **Component**: AWS SCP/GCP Org Policy
- **Artifact**: infra/cloud/guardrails/
- **Test Category**: Policy Tests
- **Metric/KPI**: # policy bypass attempts
- **Evidence**: Cloud policy reports

### Compute Images
- **Control**: Hardened AMIs/Images
- **Description**: Golden images pipeline
- **Component**: Packer, AMI bake
- **Artifact**: infra/images/packer.json
- **Test Category**: Image Tests
- **Metric/KPI**: Drift from baseline
- **Evidence**: Image recipe, hash

### Network
- **Control**: SG/NACL Baselines
- **Description**: Default-deny, tiered subnets
- **Component**: Terraform modules
- **Artifact**: infra/network/
- **Test Category**: Infra Tests
- **Metric/KPI**: Open ports exposure
- **Evidence**: TF plans, security scans

### Metadata
- **Control**: IMDSv2/Metadata Hardening
- **Description**: Prevent SSRF credentials theft
- **Component**: IMDSv2 enforced
- **Artifact**: infra/compute/metadata.json
- **Test Category**: Pen Tests/SSRF
- **Metric/KPI**: IMDSv2 usage %
- **Evidence**: Cloud config screenshots

## 12. Data Security

### Classification
- **Control**: Data Classes & Tags
- **Description**: Public/Int/Conf/Restricted with lineage
- **Component**: DLP tagging
- **Artifact**: data/classification.yaml
- **Test Category**: DLP Tests
- **Metric/KPI**: % assets tagged; unknown data
- **Evidence**: Tagging exports

### Encryption at Rest
- **Control**: Transparent & Field-level
- **Description**: TDE plus FLE for sensitive fields
- **Component**: KMS-integrated
- **Artifact**: db/encryption.yaml
- **Test Category**: Crypto Tests
- **Metric/KPI**: % encrypted fields; key age
- **Evidence**: Key map, schema diffs

### Masking/Tokenization
- **Control**: PII Handling
- **Description**: Live data masking in lower envs
- **Component**: masking rules
- **Artifact**: data/masking.yaml
- **Test Category**: Masking Tests
- **Metric/KPI**: % masked in non-prod
- **Evidence**: Masking rules, test logs

### Backups
- **Control**: Encrypted Backups & Restore Drills
- **Description**: Immutable backups with periodic restore tests
- **Component**: backup jobs
- **Artifact**: jobs/backup-restore.yaml
- **Test Category**: DR Drills
- **Metric/KPI**: RPO/RTO success
- **Evidence**: Restore reports, hashes

## 13. Application Security

### Input Validation
- **Control**: Strict Validation & CEI
- **Description**: Central validators; checks-effects-interactions
- **Component**: Rust type system, validators
- **Artifact**: crates/validation/
- **Test Category**: Fuzz/Unit Tests
- **Metric/KPI**: Reject rate for invalid inputs
- **Evidence**: Test logs, coverage

### Session/Web
- **Control**: CSRF, CORS, SSRF Guards
- **Description**: Defense-in-depth headers & checks
- **Component**: middlewares in Axum
- **Artifact**: crates/middleware/
- **Test Category**: DAST/Web Tests
- **Metric/KPI**: CSRF token coverage
- **Evidence**: Middleware code, configs

### Memory Safety
- **Control**: Unsafe minimization
- **Description**: Ban unsafe unless approved ADR
- **Component**: lint rules
- **Artifact**: deny.toml
- **Test Category**: Static Tests
- **Metric/KPI**: Lines of unsafe; review notes
- **Evidence**: Lint outputs, ADRs

### Invariants
- **Control**: Business Invariant Checks
- **Description**: Pre/post-conditions and asserts
- **Component**: contracts lib
- **Artifact**: crates/invariants/
- **Test Category**: Property Tests
- **Metric/KPI**: Invariant violation count
- **Evidence**: Proptest results

## 14. Protocol/API Security

### Contracts
- **Control**: OpenAPI/GraphQL/gRPC
- **Description**: Versioned, reviewed contracts
- **Component**: api/*
- **Artifact**: api/openapi.yaml
- **Test Category**: Contract Tests
- **Metric/KPI**: Breaking changes caught
- **Evidence**: Schema diffs, approvals

### Pagination & ETags
- **Control**: DoS-safe patterns
- **Description**: Limit page sizes; conditional requests
- **Component**: Axum extractors
- **Artifact**: crates/http/
- **Test Category**: Load Tests
- **Metric/KPI**: % endpoints with limits
- **Evidence**: Endpoint catalog

### Query Costing
- **Control**: Cost Limits/Complexity
- **Description**: GraphQL cost/Depth limits; SQL guards
- **Component**: graphql rules; SQL guards
- **Artifact**: api/graphql/policy.json
- **Test Category**: Abuse Tests
- **Metric/KPI**: Rejected costly queries
- **Evidence**: Policy files, logs

### Schema Diffs
- **Control**: Automated Drift Detection
- **Description**: PR-time diff & changelogs
- **Component**: CI job
- **Artifact**: ci/schema-diff.yaml
- **Test Category**: CI Tests
- **Metric/KPI**: # drift incidents
- **Evidence**: Diff reports

## 15. Messaging & Event Security

### AuthN/Z
- **Control**: NATS/JetStream ACLs
- **Description**: Per-subject tokens/permissions
- **Component**: nats server config
- **Artifact**: nats/auth.conf
- **Test Category**: AuthZ Tests
- **Metric/KPI**: Denied unauthorized subs/pubs
- **Evidence**: Broker logs

### Idempotency
- **Control**: Idempotency Keys & Dedupe
- **Description**: Handle retries safely
- **Component**: JetStream KV/Sequence
- **Artifact**: crates/idempotency/
- **Test Category**: Chaos/Retry Tests
- **Metric/KPI**: Duplication rate
- **Evidence**: Idempotency tables/logs

### Replay Protection
- **Control**: Nonces/Timestamps
- **Description**: Reject stale messages
- **Component**: middleware
- **Artifact**: crates/middleware/
- **Test Category**: Security Tests
- **Metric/KPI**: Replay reject count
- **Evidence**: Timestamp checks

### Encryption
- **Control**: At-rest & in-transit
- **Description**: mTLS + payload encryption if required
- **Component**: nkey/JWT
- **Artifact**: nats/nkey/
- **Test Category**: Crypto Tests
- **Metric/KPI**: Encrypted subjects ratio
- **Evidence**: Config, key materials

## 16. Database Security

### RBAC
- **Control**: DB Roles & Least Privilege
- **Description**: Per-service roles, read vs write separation
- **Component**: SQL migrations
- **Artifact**: db/migrations/roles.sql
- **Test Category**: Privilege Tests
- **Metric/KPI**: # superuser grants (0)
- **Evidence**: Role grants, audit

### RLS/CLS
- **Control**: Row/Column-level Security
- **Description**: Policies per tenant/user
- **Component**: Postgres RLS, MySQL views
- **Artifact**: db/rls.sql
- **Test Category**: Policy Tests
- **Metric/KPI**: % tables with RLS
- **Evidence**: RLS policies, tests

### TLS & Audit
- **Control**: Encrypted Links & Audit
- **Description**: TLS required; audit logs shipped
- **Component**: db configs
- **Artifact**: db/postgresql.conf
- **Test Category**: Config Tests
- **Metric/KPI**: % TLS connections; audit volume
- **Evidence**: TLS params, audit logs

### Backups/Restore
- **Control**: Encrypted Backups
- **Description**: Hot/cold backups with drills
- **Component**: pgbackrest/xbstream
- **Artifact**: backup/
- **Test Category**: DR Tests
- **Metric/KPI**: RPO/RTO achieved
- **Evidence**: Restore test logs

## 17. Wallet/Custody & Key Ops (Web3)

### Tiers
- **Control**: Hot/Warm/Cold Segregation
- **Description**: Withdrawals via tiered paths
- **Component**: Custody service
- **Artifact**: custody/policy.yaml
- **Test Category**: Flow Tests
- **Metric/KPI**: % volume via hot; risk caps
- **Evidence**: Approval logs

### MPC/Multi-sig
- **Control**: Threshold Signing
- **Description**: Split signing across parties/devices
- **Component**: MPC engine
- **Artifact**: custody/mpc/
- **Test Category**: Crypto Tests
- **Metric/KPI**: Signer availability; quorum success
- **Evidence**: Sig transcripts

### Withdrawal Policy
- **Control**: Velocity/Cooldowns
- **Description**: Per-user limits, cooling periods
- **Component**: policy engine
- **Artifact**: policies/withdrawal.yaml
- **Test Category**: Abuse Tests
- **Metric/KPI**: Fraud catches; false positives
- **Evidence**: Policy decisions

### Address Mgmt
- **Control**: Allow/Block Lists
- **Description**: Sanctions, scam list checks
- **Component**: screening service
- **Artifact**: custody/screen.yaml
- **Test Category**: Compliance Tests
- **Metric/KPI**: # blocked withdrawals
- **Evidence**: Screen results, proofs

## 18. Oracle & Market Data Integrity (Web3)

### Aggregation
- **Control**: TWAP/Medianizers
- **Description**: Robust aggregation of prices
- **Component**: oracle service
- **Artifact**: oracle/aggregator.yaml
- **Test Category**: Model Tests
- **Metric/KPI**: Deviation vs spot
- **Evidence**: TWAP windows, charts

### Heartbeat/Staleness
- **Control**: Max Age Policies
- **Description**: Reject stale feeds
- **Component**: staleness guards
- **Artifact**: oracle/heartbeat.yaml
- **Test Category**: Staleness Tests
- **Metric/KPI**: % updates within SLA
- **Evidence**: Timestamps, alarms

### Deviation Guards
- **Control**: Max % Change
- **Description**: Pause or circuit on spikes
- **Component**: risk engine
- **Artifact**: oracle/deviation.yaml
- **Test Category**: Chaos/Spike Tests
- **Metric/KPI**: # deviations triggered
- **Evidence**: Guard events

### Quorum
- **Control**: Cross-source Voting
- **Description**: Min N-of-M sources to accept
- **Component**: quorum config
- **Artifact**: oracle/quorum.yaml
- **Test Category**: Byzantine Tests
- **Metric/KPI**: # times quorum failed
- **Evidence**: Quorum decisions

## 19. Privacy & Compliance

### Minimization
- **Control**: Collect-Only-What-Needed
- **Description**: PIA reviews for new data
- **Component**: PIA templates
- **Artifact**: privacy/PIA.md
- **Test Category**: PIA Tests
- **Metric/KPI**: PIA coverage ratio
- **Evidence**: PIA records

### DSR Flows
- **Control**: Access/Delete/Export
- **Description**: Authenticated self-serve portal
- **Component**: privacy service
- **Artifact**: privacy/dsr.yaml
- **Test Category**: Functional Tests
- **Metric/KPI**: DSR SLA met
- **Evidence**: DSR tickets, proofs

### Retention
- **Control**: Time-bound Deletion
- **Description**: TTL-based auto-delete
- **Component**: db TTL jobs
- **Artifact**: data/retention.yaml
- **Test Category**: Retention Tests
- **Metric/KPI**: % data beyond TTL (0)
- **Evidence**: Deletion logs

### Regionalization
- **Control**: Data Residency
- **Description**: Store/process in-region
- **Component**: Geo shards
- **Artifact**: infra/geo/
- **Test Category**: Geo Tests
- **Metric/KPI**: Cross-region accesses blocked
- **Evidence**: Region maps, configs

## 20. Observability & Telemetry Security

### Structured Logging
- **Control**: PII Scrubbing
- **Description**: No secrets/PII; redaction
- **Component**: OTel processors
- **Artifact**: otel/redaction.yaml
- **Test Category**: Log Tests
- **Metric/KPI**: Redaction hit/miss
- **Evidence**: Processor configs, samples

### Metrics/Traces Access
- **Control**: RBAC on Observability
- **Description**: Limit who can see raw traces
- **Component**: Grafana/Tempo perms
- **Artifact**: observability/rbac.yaml
- **Test Category**: Access Tests
- **Metric/KPI**: # unauthorized attempts
- **Evidence**: Audit logs

### Tamper Evidence
- **Control**: Immutable Logs
- **Description**: WORM storage or hash-chains
- **Component**: Loki object store
- **Artifact**: observability/tamper.yaml
- **Test Category**: Integrity Tests
- **Metric/KPI**: Hash-chain breaks (0)
- **Evidence**: Hashes, receipts

### Trace Sampling
- **Control**: Dynamic Sampling
- **Description**: Protect cost & privacy
- **Component**: collector rules
- **Artifact**: otel/sampling.yaml
- **Test Category**: Perf Tests
- **Metric/KPI**: Sampling % adherence
- **Evidence**: Rules, dashboards

## 21. Detection & Response

### Use Cases
- **Control**: SIEM Rules & Detections
- **Description**: Mapped to ATT&CK and env specifics
- **Component**: Sigma rules
- **Artifact**: siem/rules/
- **Test Category**: Detection QA
- **Metric/KPI**: True/false positive rates
- **Evidence**: Rule packs, test results

### Anomaly Models
- **Control**: UEBA/Anomaly
- **Description**: Behavioral baselines, alerts
- **Component**: ML jobs
- **Artifact**: siem/ueba/
- **Test Category**: Model Tests
- **Metric/KPI**: Alert precision/recall
- **Evidence**: Model evals

### Runbooks
- **Control**: IR Runbooks & On-call
- **Description**: Per alert type playbooks
- **Component**: docs/runbooks/
- **Artifact**: docs/runbooks/*
- **Test Category**: Drill Tests
- **Metric/KPI**: MTTD/MTTR
- **Evidence**: Drill notes, timelines

### Forensics
- **Control**: Acquisition & Chain of Custody
- **Description**: Disk/mem capture, legal hold
- **Component**: Forensics kits
- **Artifact**: ir/forensics/
- **Test Category**: Forensic Drills
- **Metric/KPI**: Case completeness
- **Evidence**: Evidence manifests

## 22. Resilience, Availability & Chaos

### Rate Limiters
- **Control**: Backpressure & Quotas
- **Description**: Protect shared resources
- **Component**: leaky-bucket libs
- **Artifact**: crates/ratelimit/
- **Test Category**: Soak/Stress Tests
- **Metric/KPI**: Shed load %
- **Evidence**: Config, graphs

### Circuit Breakers
- **Control**: Fail-Fast
- **Description**: Trip on dependency failures
- **Component**: tower::limit/middleware
- **Artifact**: crates/circuit/
- **Test Category**: Chaos/Latency Tests
- **Metric/KPI**: Trip frequency; recovery time
- **Evidence**: Breaker state logs

### Bulkheads
- **Control**: Isolate Pools
- **Description**: Isolate threads/connections
- **Component**: tokio pools
- **Artifact**: crates/bulkhead/
- **Test Category**: Stress Tests
- **Metric/KPI**: Blast radius size
- **Evidence**: Pool configs

### DR/BCP
- **Control**: Disaster Recovery
- **Description**: Multi-region, restore, exercises
- **Component**: runbooks/dr/
- **Artifact**: docs/dr/PLAN.md
- **Test Category**: DR Drills
- **Metric/KPI**: RPO/RTO met
- **Evidence**: Exercise reports