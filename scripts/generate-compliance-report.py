#!/usr/bin/env python3
"""
Script to generate compliance reports based on the security layers checklist.
"""

import csv
import json
import os
import sys
from datetime import datetime
from typing import Dict, List, Any

def load_checklist(filename: str) -> List[Dict[str, Any]]:
    """Load the security layers checklist from CSV file."""
    checklist = []
    try:
        with open(filename, 'r', newline='', encoding='utf-8') as csvfile:
            reader = csv.DictReader(csvfile)
            for row in reader:
                checklist.append(row)
    except FileNotFoundError:
        print(f"Error: Checklist file '{filename}' not found.")
        sys.exit(1)
    except Exception as e:
        print(f"Error reading checklist: {e}")
        sys.exit(1)
    
    return checklist

def check_artifact_exists(artifact_path: str) -> bool:
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

def assess_control(control: Dict[str, Any]) -> Dict[str, Any]:
    """Assess a single control and return assessment results."""
    artifact = control.get("Policy/Config Artifact", "")
    
    # Determine status
    if not artifact:
        status = "Not Applicable"
        reason = "No artifact specified"
    elif check_artifact_exists(artifact):
        status = "Implemented"
        reason = "Artifact found"
    else:
        status = "Missing"
        reason = "Artifact not found"
    
    return {
        "layer_number": control.get("Layer #", ""),
        "layer_name": control.get("Layer Name", ""),
        "control_group": control.get("Control Group", ""),
        "control": control.get("Control", ""),
        "status": status,
        "reason": reason,
        "artifact": artifact,
        "component": control.get("Component (Rust/K8s/Web3)", ""),
        "test_category": control.get("Test Category", ""),
        "metric_kpi": control.get("Metric/KPI", ""),
        "evidence": control.get("Evidence to Store", "")
    }

def generate_summary(assessments: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate summary statistics from assessments."""
    total = len(assessments)
    implemented = sum(1 for a in assessments if a["status"] == "Implemented")
    missing = sum(1 for a in assessments if a["status"] == "Missing")
    not_applicable = sum(1 for a in assessments if a["status"] == "Not Applicable")
    
    # Calculate compliance rate
    applicable = implemented + missing
    compliance_rate = (implemented / applicable * 100) if applicable > 0 else 0
    
    # Group by layer
    layer_stats = {}
    for assessment in assessments:
        layer_num = assessment["layer_number"]
        if layer_num not in layer_stats:
            layer_stats[layer_num] = {"implemented": 0, "missing": 0, "not_applicable": 0}
        
        if assessment["status"] == "Implemented":
            layer_stats[layer_num]["implemented"] += 1
        elif assessment["status"] == "Missing":
            layer_stats[layer_num]["missing"] += 1
        elif assessment["status"] == "Not Applicable":
            layer_stats[layer_num]["not_applicable"] += 1
    
    return {
        "generated_at": datetime.now().isoformat(),
        "total_controls": total,
        "implemented": implemented,
        "missing": missing,
        "not_applicable": not_applicable,
        "applicable": applicable,
        "compliance_rate": compliance_rate,
        "layer_statistics": layer_stats
    }

def generate_text_report(assessments: List[Dict[str, Any]], summary: Dict[str, Any]) -> str:
    """Generate a text report."""
    report = []
    report.append("SECURITY COMPLIANCE REPORT")
    report.append("=" * 50)
    report.append(f"Generated: {summary['generated_at']}")
    report.append("")
    
    # Summary
    report.append("SUMMARY")
    report.append("-" * 20)
    report.append(f"Total Controls: {summary['total_controls']}")
    report.append(f"Implemented: {summary['implemented']}")
    report.append(f"Missing: {summary['missing']}")
    report.append(f"Not Applicable: {summary['not_applicable']}")
    report.append(f"Compliance Rate: {summary['compliance_rate']:.2f}%")
    report.append("")
    
    # Layer statistics
    report.append("LAYER STATISTICS")
    report.append("-" * 20)
    for layer_num, stats in summary["layer_statistics"].items():
        total_layer = stats["implemented"] + stats["missing"] + stats["not_applicable"]
        applicable_layer = stats["implemented"] + stats["missing"]
        rate = (stats["implemented"] / applicable_layer * 100) if applicable_layer > 0 else 0
        report.append(f"Layer {layer_num}: {stats['implemented']}/{applicable_layer} ({rate:.1f}%)")
    report.append("")
    
    # Missing controls
    missing_controls = [a for a in assessments if a["status"] == "Missing"]
    if missing_controls:
        report.append("MISSING CONTROLS")
        report.append("-" * 20)
        for control in missing_controls:
            report.append(f"Layer {control['layer_number']}: {control['control_group']} - {control['control']}")
            report.append(f"  Artifact: {control['artifact']}")
            report.append("")
    
    return "\n".join(report)

def generate_json_report(assessments: List[Dict[str, Any]], summary: Dict[str, Any]) -> str:
    """Generate a JSON report."""
    report = {
        "summary": summary,
        "assessments": assessments
    }
    return json.dumps(report, indent=2)

def main():
    """Main function."""
    # Check if checklist file exists
    checklist_file = "security_layers_checklist.csv"
    if not os.path.exists(checklist_file):
        print(f"Error: Checklist file '{checklist_file}' not found.")
        sys.exit(1)
    
    # Load checklist
    checklist = load_checklist(checklist_file)
    
    # Assess each control
    assessments = []
    for control in checklist:
        assessment = assess_control(control)
        assessments.append(assessment)
    
    # Generate summary
    summary = generate_summary(assessments)
    
    # Generate reports
    text_report = generate_text_report(assessments, summary)
    json_report = generate_json_report(assessments, summary)
    
    # Save reports
    with open("security-compliance-report.txt", "w", encoding="utf-8") as f:
        f.write(text_report)
    
    with open("security-compliance-report.json", "w", encoding="utf-8") as f:
        f.write(json_report)
    
    # Print summary to console
    print(text_report)
    
    print("\nReports saved:")
    print("- security-compliance-report.txt")
    print("- security-compliance-report.json")

if __name__ == "__main__":
    main()