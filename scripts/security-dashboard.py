#!/usr/bin/env python3
"""
Security Dashboard Generator

This script generates a security dashboard with metrics and KPIs
based on the security layers checklist.
"""

import csv
import json
import os
import subprocess
import sys
from datetime import datetime
from typing import Dict, List, Any

class SecurityDashboard:
    def __init__(self, checklist_file: str = "security_layers_checklist.csv"):
        self.checklist_file = checklist_file
        self.checklist = self.load_checklist()
        self.metrics = {}
        
    def load_checklist(self) -> List[Dict[str, Any]]:
        """Load the security layers checklist from CSV file."""
        checklist = []
        try:
            with open(self.checklist_file, 'r', newline='', encoding='utf-8') as csvfile:
                reader = csv.DictReader(csvfile)
                for row in reader:
                    checklist.append(row)
        except FileNotFoundError:
            print(f"Error: Checklist file '{self.checklist_file}' not found.")
            sys.exit(1)
        except Exception as e:
            print(f"Error reading checklist: {e}")
            sys.exit(1)
        
        return checklist
    
    def run_security_audit(self) -> Dict[str, Any]:
        """Run security audit tools and collect metrics."""
        metrics = {
            "timestamp": datetime.now().isoformat(),
            "vulnerabilities": {},
            "compliance": {},
            "dependencies": {},
            "secrets": {}
        }
        
        # Run cargo audit if available
        try:
            result = subprocess.run(
                ["cargo", "audit", "--json"],
                capture_output=True,
                text=True,
                timeout=60
            )
            if result.returncode == 0:
                audit_data = json.loads(result.stdout)
                metrics["vulnerabilities"]["count"] = len(audit_data.get("vulnerabilities", []))
                metrics["vulnerabilities"]["critical"] = len([
                    v for v in audit_data.get("vulnerabilities", [])
                    if v.get("severity") == "critical"
                ])
            else:
                metrics["vulnerabilities"]["error"] = "Cargo audit failed"
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, FileNotFoundError):
            metrics["vulnerabilities"]["error"] = "Cargo audit not available"
        
        # Run cargo deny checks
        try:
            result = subprocess.run(
                ["cargo", "deny", "check", "--format", "json"],
                capture_output=True,
                text=True,
                timeout=60
            )
            if result.returncode == 0:
                deny_data = json.loads(result.stdout)
                metrics["compliance"]["license_issues"] = len(deny_data.get("licenses", []))
                metrics["compliance"]["ban_issues"] = len(deny_data.get("bans", []))
            else:
                metrics["compliance"]["error"] = "Cargo deny failed"
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, FileNotFoundError):
            metrics["compliance"]["error"] = "Cargo deny not available"
        
        return metrics
    
    def calculate_compliance_metrics(self) -> Dict[str, Any]:
        """Calculate compliance metrics from the checklist."""
        total_controls = len(self.checklist)
        implemented_controls = 0
        missing_controls = 0
        
        # Count implemented vs missing controls
        for control in self.checklist:
            artifact = control.get("Policy/Config Artifact", "")
            if artifact and self.check_artifact_exists(artifact):
                implemented_controls += 1
            elif artifact:
                missing_controls += 1
        
        # Calculate by layer
        layer_metrics = {}
        for control in self.checklist:
            layer_num = control.get("Layer #", "Unknown")
            if layer_num not in layer_metrics:
                layer_metrics[layer_num] = {"total": 0, "implemented": 0, "missing": 0}
            
            layer_metrics[layer_num]["total"] += 1
            artifact = control.get("Policy/Config Artifact", "")
            if artifact and self.check_artifact_exists(artifact):
                layer_metrics[layer_num]["implemented"] += 1
            elif artifact:
                layer_metrics[layer_num]["missing"] += 1
        
        return {
            "total_controls": total_controls,
            "implemented_controls": implemented_controls,
            "missing_controls": missing_controls,
            "compliance_rate": (implemented_controls / total_controls * 100) if total_controls > 0 else 0,
            "layer_metrics": layer_metrics
        }
    
    def check_artifact_exists(self, artifact_path: str) -> bool:
        """Check if an artifact (file or directory) exists."""
        if not artifact_path:
            return False
        
        # Handle wildcard patterns
        if '*' in artifact_path or '?' in artifact_path:
            import glob
            matches = glob.glob(artifact_path, recursive=True)
            return len(matches) > 0
        
        # Check exact path
        return os.path.exists(artifact_path)
    
    def generate_html_report(self, audit_metrics: Dict[str, Any], compliance_metrics: Dict[str, Any]) -> str:
        """Generate an HTML dashboard report."""
        html = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Security Dashboard</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .metrics {{ display: flex; flex-wrap: wrap; gap: 20px; margin: 20px 0; }}
        .metric-card {{ background-color: #f9f9f9; padding: 15px; border-radius: 5px; min-width: 200px; }}
        .metric-value {{ font-size: 24px; font-weight: bold; }}
        .chart-container {{ margin: 20px 0; }}
        .layer-table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        .layer-table th, .layer-table td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        .layer-table th {{ background-color: #f2f2f2; }}
        .compliant {{ color: green; }}
        .non-compliant {{ color: red; }}
        .warning {{ color: orange; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Security Dashboard</h1>
        <p>Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
    </div>
    
    <div class="metrics">
        <div class="metric-card">
            <h3>Overall Compliance</h3>
            <div class="metric-value">{compliance_metrics['compliance_rate']:.1f}%</div>
            <p>{compliance_metrics['implemented_controls']}/{compliance_metrics['total_controls']} controls implemented</p>
        </div>
        
        <div class="metric-card">
            <h3>Vulnerabilities</h3>
            <div class="metric-value">{audit_metrics['vulnerabilities'].get('count', 'N/A')}</div>
            <p>{audit_metrics['vulnerabilities'].get('critical', 0)} critical</p>
        </div>
        
        <div class="metric-card">
            <h3>Missing Controls</h3>
            <div class="metric-value">{compliance_metrics['missing_controls']}</div>
            <p>controls need implementation</p>
        </div>
    </div>
    
    <div class="chart-container">
        <h2>Compliance by Security Layer</h2>
        <table class="layer-table">
            <thead>
                <tr>
                    <th>Layer #</th>
                    <th>Layer Name</th>
                    <th>Compliance Rate</th>
                    <th>Implemented</th>
                    <th>Total</th>
                </tr>
            </thead>
            <tbody>
"""
        
        # Add layer metrics
        for layer_num, metrics in compliance_metrics["layer_metrics"].items():
            rate = (metrics["implemented"] / metrics["total"] * 100) if metrics["total"] > 0 else 0
            status_class = "compliant" if rate >= 90 else "warning" if rate >= 70 else "non-compliant"
            
            html += f"""
                <tr>
                    <td>{layer_num}</td>
                    <td>{self.get_layer_name(layer_num)}</td>
                    <td class="{status_class}">{rate:.1f}%</td>
                    <td>{metrics['implemented']}</td>
                    <td>{metrics['total']}</td>
                </tr>
"""
        
        html += """
            </tbody>
        </table>
    </div>
    
    <div class="chart-container">
        <h2>Security Metrics</h2>
        <h3>Vulnerability Assessment</h3>
        <p>Critical Vulnerabilities: {audit_metrics['vulnerabilities'].get('critical', 0)}</p>
        <p>Total Vulnerabilities: {audit_metrics['vulnerabilities'].get('count', 'N/A')}</p>
        
        <h3>Compliance Issues</h3>
        <p>License Issues: {audit_metrics['compliance'].get('license_issues', 'N/A')}</p>
        <p>Ban Issues: {audit_metrics['compliance'].get('ban_issues', 'N/A')}</p>
    </div>
</body>
</html>
"""
        return html
    
    def get_layer_name(self, layer_num: str) -> str:
        """Get the name of a security layer by its number."""
        layer_names = {
            "1": "Governance & Policy",
            "2": "Risk & Threat Modeling",
            "3": "Secure SDLC & Supply Chain",
            "4": "Identity & Access (IAM)",
            "5": "Secrets Management",
            "6": "Key & Cryptography",
            "7": "Network Segmentation & Transport",
            "8": "Perimeter & API Gateway",
            "9": "Host/Endpoint Hardening",
            "10": "Containers & Orchestration",
            "11": "Cloud/IaaS Security",
            "12": "Data Security",
            "13": "Application Security",
            "14": "Protocol/API Security",
            "15": "Messaging & Event Security",
            "16": "Database Security",
            "17": "Wallet/Custody & Key Ops (Web3)",
            "18": "Oracle & Market Data Integrity (Web3)",
            "19": "Privacy & Compliance",
            "20": "Observability & Telemetry Security",
            "21": "Detection & Response",
            "22": "Resilience, Availability & Chaos"
        }
        return layer_names.get(layer_num, "Unknown Layer")
    
    def generate_dashboard(self):
        """Generate the complete security dashboard."""
        print("Generating security dashboard...")
        
        # Run security audits
        audit_metrics = self.run_security_audit()
        
        # Calculate compliance metrics
        compliance_metrics = self.calculate_compliance_metrics()
        
        # Generate HTML report
        html_report = self.generate_html_report(audit_metrics, compliance_metrics)
        
        # Save reports
        with open("security-dashboard.html", "w", encoding="utf-8") as f:
            f.write(html_report)
        
        # Save metrics as JSON
        dashboard_data = {
            "timestamp": datetime.now().isoformat(),
            "audit_metrics": audit_metrics,
            "compliance_metrics": compliance_metrics
        }
        
        with open("security-dashboard.json", "w", encoding="utf-8") as f:
            json.dump(dashboard_data, f, indent=2)
        
        print("Security dashboard generated successfully!")
        print("- security-dashboard.html")
        print("- security-dashboard.json")
        
        # Print summary
        print(f"\nDashboard Summary:")
        print(f"Compliance Rate: {compliance_metrics['compliance_rate']:.1f}%")
        print(f"Implemented Controls: {compliance_metrics['implemented_controls']}/{compliance_metrics['total_controls']}")
        print(f"Missing Controls: {compliance_metrics['missing_controls']}")
        print(f"Critical Vulnerabilities: {audit_metrics['vulnerabilities'].get('critical', 0)}")

def main():
    """Main function."""
    dashboard = SecurityDashboard()
    dashboard.generate_dashboard()

if __name__ == "__main__":
    main()