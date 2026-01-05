#!/usr/bin/env python3
"""
TruffleHog secret scanning for Git pre-commit hook

Scans only staged files for secrets before allowing commit.
Uses TruffleHog with --only-verified to reduce false positives.
"""

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import List, Set

# File extensions to scan (Rust project focus)
SCANNED_EXTENSIONS = {
    ".rs",      # Rust source
    ".toml",    # Cargo config
    ".json",    # JSON config
    ".yml",     # YAML config
    ".yaml",    # YAML config
    ".sh",      # Shell scripts
    ".bash",    # Bash scripts
    ".txt",     # Text files
    ".md",      # Markdown (may contain examples)
    ".env",     # Environment files
}

# Maximum file size to scan (1MB)
MAX_FILE_SIZE = 1024 * 1024

def get_repo_root() -> Path:
    """Get the repository root directory"""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            check=True
        )
        return Path(result.stdout.strip())
    except subprocess.CalledProcessError:
        return Path.cwd()

def get_staged_files() -> List[Path]:
    """Get list of staged files for commit"""
    try:
        result = subprocess.run(
            ["git", "diff", "--cached", "--name-only", "--diff-filter=ACM"],
            capture_output=True,
            text=True,
            check=True
        )
        repo_root = get_repo_root()
        files = []
        for line in result.stdout.strip().split("\n"):
            if line:
                file_path = repo_root / line
                if file_path.exists():
                    files.append(file_path)
        return files
    except subprocess.CalledProcessError:
        return []

def should_scan_file(file_path: Path) -> bool:
    """Determine if file should be scanned"""
    # Check extension
    if file_path.suffix not in SCANNED_EXTENSIONS:
        return False

    # Check file size
    try:
        if file_path.stat().st_size > MAX_FILE_SIZE:
            return False
    except OSError:
        return False

    return True

def load_allowlist(repo_root: Path) -> Set[str]:
    """Load allowlist of permitted secrets from allow.json"""
    allowlist_file = repo_root / "allow.json"
    if not allowlist_file.exists():
        return set()

    try:
        with open(allowlist_file) as f:
            data = json.load(f)
            # Extract all hashes from allowlist
            hashes = set()
            for entry in data.get("allowlist", []):
                if "hash" in entry:
                    hashes.add(entry["hash"])
            return hashes
    except (json.JSONDecodeError, IOError):
        return set()

def run_trufflehog(files: List[Path], repo_root: Path) -> bool:
    """Run TruffleHog on specified files"""
    if not files:
        return True  # No files to scan

    # Create temporary file list
    temp_file = repo_root / ".trufflehog_scan_list.tmp"
    try:
        with open(temp_file, "w") as f:
            for file_path in files:
                f.write(f"{file_path}\n")

        # Build TruffleHog command
        cmd = [
            "trufflehog",
            "filesystem",
            "--only-verified",
            "--json",
        ]

        # Add exclude paths if exists
        exclude_file = repo_root / ".trufflehog_exclude.txt"
        if exclude_file.exists():
            cmd.extend(["--exclude-paths", str(exclude_file)])

        # Scan each file individually to respect file list
        all_findings = []
        for file_path in files:
            result = subprocess.run(
                cmd + [str(file_path)],
                capture_output=True,
                text=True
            )

            if result.stdout:
                # Parse JSON output
                for line in result.stdout.strip().split("\n"):
                    if line:
                        try:
                            finding = json.loads(line)
                            all_findings.append(finding)
                        except json.JSONDecodeError:
                            continue

        # Filter against allowlist
        allowlist = load_allowlist(repo_root)
        blocked_findings = []

        for finding in all_findings:
            # Generate hash for this finding (simple approach)
            raw = finding.get("Raw", "")
            finding_hash = str(hash(raw))

            if finding_hash not in allowlist:
                blocked_findings.append(finding)

        if blocked_findings:
            print("\n❌ SECRET SCAN FAILED", file=sys.stderr)
            print("=" * 60, file=sys.stderr)
            print(f"Found {len(blocked_findings)} verified secret(s) in staged files:\n", file=sys.stderr)

            for i, finding in enumerate(blocked_findings, 1):
                detector = finding.get("DetectorName", "Unknown")
                file_path = finding.get("SourceMetadata", {}).get("Data", {}).get("Filesystem", {}).get("file", "Unknown")
                print(f"{i}. {detector} in {file_path}", file=sys.stderr)

            print("\n" + "=" * 60, file=sys.stderr)
            print("To fix:", file=sys.stderr)
            print("  1. Remove the secrets from your code", file=sys.stderr)
            print("  2. Use environment variables or secure vaults instead", file=sys.stderr)
            print("  3. If this is a false positive, add to allow.json", file=sys.stderr)
            print("\nTo bypass (NOT recommended):", file=sys.stderr)
            print("  git commit --no-verify", file=sys.stderr)
            return False

        return True

    finally:
        # Clean up temp file
        if temp_file.exists():
            temp_file.unlink()

def main() -> int:
    """Main entry point"""
    repo_root = get_repo_root()

    # Check if trufflehog is installed
    try:
        subprocess.run(
            ["trufflehog", "--version"],
            capture_output=True,
            check=True
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("⚠️  TruffleHog not installed, skipping secret scan", file=sys.stderr)
        print("Install with: brew install trufflehog (macOS) or see https://github.com/trufflesecurity/trufflehog", file=sys.stderr)
        return 0

    # Get staged files
    staged_files = get_staged_files()
    if not staged_files:
        return 0  # No files to scan

    # Filter to scannable files
    files_to_scan = [f for f in staged_files if should_scan_file(f)]

    if not files_to_scan:
        return 0  # No relevant files

    print(f"🔍 Scanning {len(files_to_scan)} file(s) for secrets...", file=sys.stderr)

    # Run scan
    if run_trufflehog(files_to_scan, repo_root):
        print("✅ No secrets detected", file=sys.stderr)
        return 0
    else:
        return 1

if __name__ == "__main__":
    sys.exit(main())
