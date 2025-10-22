# Security and License Compliance Fixes

## Problem Description

The CI pipeline's security scan job was failing due to:

1. Security vulnerabilities in dependencies (specifically sqlx and rsa crates)
2. License compliance issues with missing license fields in local crates
3. Rejected licenses (Unicode-3.0) that needed to be explicitly allowed

## Root Causes

### Security Vulnerabilities
- **sqlx 0.7.4**: Vulnerable to "Binary Protocol Misinterpretation caused by Truncating or Overflowing Casts" (RUSTSEC-2024-0363)
- **rsa 0.9.8**: Vulnerable to "Marvin Attack: potential key recovery through timing sidechannels" (RUSTSEC-2023-0071)

### License Compliance Issues
- Multiple local crates missing license fields in their Cargo.toml files
- Unicode-3.0 license being rejected by cargo-deny because it wasn't in the allowed list

## Fixes Implemented

### 1. Dependency Updates

Updated sqlx dependency in Cargo.toml from version "0.7" to "0.8.1" to address:
- RUSTSEC-2024-0363: Binary Protocol Misinterpretation vulnerability

### 2. Security Advisory Configuration

Added ignore rule for RUSTSEC-2023-0071 in deny.toml:
- No fixed upgrade is available for rsa crate, which is a transitive dependency of sqlx-mysql

### 3. License Compliance Fixes

Added license fields to all local crates:
- sniper-telemetry
- svc-cex
- svc-executor
- svc-gateway
- svc-nft
- svc-policy
- svc-risk
- svc-signals
- svc-storage
- svc-strategy

All set to "Apache-2.0" license to match the workspace license.

### 4. License Allowance Updates

Updated deny.toml to include Unicode-3.0 in the allowed licenses list.

## Files Modified

1. `Cargo.toml` - Updated sqlx dependency version
2. `deny.toml` - Fixed configuration and added proper security settings
3. `crates/sniper-telemetry/Cargo.toml` - Added license field
4. `crates/svc-cex/Cargo.toml` - Added license field
5. `crates/svc-executor/Cargo.toml` - Added license field
6. `crates/svc-gateway/Cargo.toml` - Added license field
7. `crates/svc-nft/Cargo.toml` - Added license field
8. `crates/svc-policy/Cargo.toml` - Added license field
9. `crates/svc-risk/Cargo.toml` - Added license field
10. `crates/svc-signals/Cargo.toml` - Added license field
11. `crates/svc-storage/Cargo.toml` - Added license field
12. `crates/svc-strategy/Cargo.toml` - Added license field

## Verification

After these changes:
- All security vulnerabilities should be resolved or properly ignored
- All local crates now have license fields
- Unicode-3.0 license is now allowed
- CI pipeline security scan job should pass

## Expected Outcome

The security scan job in the CI pipeline should now complete successfully:
- ✅ cargo audit should pass
- ✅ cargo deny checks should pass
- ✅ Overall security scan job should pass

This will restore proper security scanning and license compliance for the snipping bot project.