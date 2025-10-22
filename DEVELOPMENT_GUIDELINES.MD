# Snipping Bot Development Guidelines

## Design Phase
Define permitted strategies, risk rules, and compliance constraints before implementation begins.

## Data Layer
Build robust market data ingestion with redundant RPCs to ensure data availability and reliability.

## Strategy Sandbox
Implement strategy simulators and backtesting capabilities before live deployment.

## Execution Stubs
Build safe connectors to exchanges using testnets for validation before mainnet deployment.

## Key Management
Integrate HSM or Vault solutions and enforce least privilege access principles for all services.

## Monitoring & Alarms
Deploy comprehensive metrics collection and audit logging before handling any real funds.

## Gradual Rollout
Follow a phased approach:
- Shadow-mode trading (observation only)
- Paper trading (simulated with real market data)
- Small capital deployment
- Gradual scaling based on performance

## Security Review
Conduct both internal and third-party penetration testing before production deployment.

## Compliance Review
Perform legal checks for target jurisdictions and exchange Terms of Service (TOS) compliance.

## Operations Runbook
Prepare comprehensive operational procedures including:
- Kill switches for emergency situations
- Key compromise response plan
- System restore and recovery steps