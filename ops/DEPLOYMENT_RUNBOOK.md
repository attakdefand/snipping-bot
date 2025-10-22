# Deployment Runbook

This document provides detailed procedures for deploying the Snipping Bot system.

## Pre-Deployment Checklist

### Environment Verification

- [ ] Verify all required environment variables are set
- [ ] Confirm access to blockchain RPC endpoints
- [ ] Validate secrets and key configurations
- [ ] Check Docker and Docker Compose versions
- [ ] Verify sufficient system resources (CPU, memory, disk)

### Configuration Review

- [ ] Review `configs/chains/` for correct network configurations
- [ ] Verify `configs/strategies/` for active trading strategies
- [ ] Check `configs/risk.toml` for current risk parameters
- [ ] Confirm `configs/routes.toml` for service routing
- [ ] Validate `configs/secrets.example.toml` structure

### Service Dependencies

- [ ] Ensure all required services are available
- [ ] Verify database connectivity (if applicable)
- [ ] Check message queue connectivity
- [ ] Confirm external API access (if applicable)

## Deployment Steps

### 1. Preparation

```bash
# Clone or update repository
git pull origin main

# Navigate to project directory
cd snipping-bot

# Create backup of current configuration
cp -r configs configs.backup.$(date +%Y%m%d_%H%M%S)
```

### 2. Configuration Update

```bash
# Update environment variables
cp .env.example .env
# Edit .env with production values

# Review and update configuration files
# Edit files in configs/ as needed
```

### 3. Build Services

```bash
# Build all services
docker-compose build

# Or build specific services if updating only certain components
docker-compose build svc-strategy svc-executor
```

### 4. Service Deployment

```bash
# Stop current services
docker-compose down

# Deploy new services
docker-compose up -d

# Or deploy specific services
docker-compose up -d svc-strategy svc-executor
```

### 5. Health Verification

```bash
# Check service status
docker-compose ps

# View service logs
docker-compose logs -f svc-strategy

# Verify health endpoints (if available)
curl http://localhost:8080/health
```

### 6. Post-Deployment Validation

- [ ] Verify all services are running
- [ ] Check service logs for errors
- [ ] Confirm connectivity to blockchain networks
- [ ] Test signal processing with sample data
- [ ] Validate trade execution (in simulation mode initially)

## Rollback Procedure

### When to Rollback

- Critical errors in production
- Performance degradation
- Security vulnerabilities
- Data corruption

### Rollback Steps

```bash
# Stop current services
docker-compose down

# Restore previous configuration (if needed)
# cp -r configs.backup.<timestamp> configs

# Deploy previous version
git checkout <previous-commit-hash>
docker-compose up -d

# Verify deployment
docker-compose ps
docker-compose logs -f
```

## Post-Deployment Monitoring

### Immediate Checks (First 30 minutes)

- [ ] Service uptime and responsiveness
- [ ] Error rates and log anomalies
- [ ] Resource utilization (CPU, memory, disk)
- [ ] Blockchain connectivity status
- [ ] Signal processing throughput

### Ongoing Monitoring (First 24 hours)

- [ ] Trade execution success rate
- [ ] Risk control triggers
- [ ] Performance metrics
- [ ] Alert notifications
- [ ] User impact (if applicable)

## Troubleshooting Common Issues

### Service Won't Start

1. Check Docker logs: `docker-compose logs <service-name>`
2. Verify environment variables
3. Check configuration file syntax
4. Ensure sufficient system resources
5. Validate network connectivity

### Blockchain Connectivity Issues

1. Verify RPC endpoint URLs
2. Check network connectivity to RPC providers
3. Validate API keys and authentication
4. Review rate limiting and quotas
5. Check firewall and security group settings

### Performance Degradation

1. Monitor system resources
2. Check for memory leaks
3. Review database performance
4. Examine message queue backlogs
5. Analyze profiling data

### Security Concerns

1. Review access logs
2. Check for unauthorized access attempts
3. Verify key and secret rotation
4. Audit configuration changes
5. Scan for vulnerabilities

## Contact Information

### Support Team

- Primary: devops@snippingbot.com
- Backup: support@snippingbot.com

### Emergency Contacts

- Infrastructure Lead: +1-555-0123
- Security Lead: +1-555-0124
- Operations Manager: +1-555-0125

### External Vendors

- Cloud Provider Support: support@cloudprovider.com
- Blockchain RPC Provider: support@rpcprovider.com
- Monitoring Service: support@monitoring.com