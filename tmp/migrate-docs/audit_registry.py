#!/usr/bin/env python3
"""Audit every locked registry version against crates.io's sparse index."""

import argparse
import concurrent.futures
import datetime
import json
from pathlib import Path
import time
import tomllib
import urllib.request


def index_path(name):
    if len(name) == 1:
        return "1/" + name
    if len(name) == 2:
        return "2/" + name
    if len(name) == 3:
        return "3/" + name[0] + "/" + name
    return name[:2] + "/" + name[2:4] + "/" + name


def audit(name, packages):
    url = "https://index.crates.io/" + index_path(name)
    for attempt in range(3):
        try:
            request = urllib.request.Request(
                url, headers={"User-Agent": "yororen-ui-registry-audit"}
            )
            with urllib.request.urlopen(request, timeout=60) as response:
                versions = {
                    entry["vers"]: entry
                    for entry in map(json.loads, response.read().decode().splitlines())
                }
            results = []
            for package in packages:
                entry = versions[package["version"]]
                results.append(
                    {
                        "name": name,
                        "version": package["version"],
                        "yanked": entry["yanked"],
                        "checksum_matches": package["checksum"] == entry["cksum"],
                        "index": url,
                    }
                )
            return results
        except Exception:
            if attempt == 2:
                raise
            time.sleep(attempt + 1)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--lock", type=Path, default=Path("Cargo.lock"))
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    packages = tomllib.loads(args.lock.read_text())["package"]
    manifest = tomllib.loads(Path("Cargo.toml").read_text())
    workspace_names = {
        tomllib.loads((Path(member) / "Cargo.toml").read_text())["package"]["name"]
        for member in manifest["workspace"]["members"]
    }
    errors = []
    grouped = {}
    for package in packages:
        source = package.get("source")
        if source is None:
            if package["name"] not in workspace_names:
                errors.append(f"Unexpected non-registry package: {package['name']}")
        elif source != "registry+https://github.com/rust-lang/crates.io-index":
            errors.append(f"Unexpected source: {package['name']} {source}")
        else:
            grouped.setdefault(package["name"], []).append(package)

    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=12) as executor:
        futures = {
            executor.submit(audit, name, locked): name
            for name, locked in grouped.items()
        }
        for future in concurrent.futures.as_completed(futures):
            try:
                results.extend(future.result())
            except Exception as error:
                errors.append(f"{futures[future]}: {error}")
    for result in results:
        if result["yanked"] or not result["checksum_matches"]:
            errors.append(f"Rejected package: {result}")
    report = {
        "checked_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "lock": str(args.lock),
        "registry_package_versions": len(results),
        "workspace_packages": len(packages) - sum(map(len, grouped.values())),
        "errors": errors,
        "packages": sorted(results, key=lambda item: (item["name"], item["version"])),
    }
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key != "packages"}))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()
