#!/usr/bin/env python3
"""Launch each macOS demo and require an actual onscreen native window."""

import datetime
import json
from pathlib import Path
import subprocess
import tempfile
import time
import tomllib


root = Path(__file__).resolve().parents[2]
output = root / "tmp/migrate-docs/native-startup"
output.mkdir(exist_ok=True)
results = []

with tempfile.TemporaryDirectory(prefix="yororen-native-startup-") as temporary:
    stage = Path(temporary)
    source = stage / "windows.swift"
    probe = stage / "windows"
    source.write_text(
        """import CoreGraphics
import Foundation
let pid = Int(CommandLine.arguments[1])!
let windows = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID)
    as? [[String: Any]] ?? []
let owned = windows.filter {
    ($0[kCGWindowOwnerPID as String] as? Int) == pid &&
    ($0[kCGWindowLayer as String] as? Int) == 0
}
let data = try JSONSerialization.data(withJSONObject: owned, options: [.sortedKeys])
print(String(data: data, encoding: .utf8)!)
"""
    )
    subprocess.run(["swiftc", str(source), "-o", str(probe)], check=True)
    members = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["members"]
    for member in members:
        if "/yororen-ui-demos/" not in member:
            continue
        name = tomllib.loads((root / member / "Cargo.toml").read_text())["package"]["name"]
        binary = root / "target/debug" / name
        log = output / (name + ".log")
        with log.open("wb") as stream:
            process = subprocess.Popen([str(binary)], cwd=root, stdout=stream, stderr=stream)
            try:
                deadline = time.monotonic() + 15
                windows = []
                while process.poll() is None and time.monotonic() < deadline:
                    windows = json.loads(
                        subprocess.check_output([str(probe), str(process.pid)], text=True)
                    )
                    if windows:
                        break
                    time.sleep(0.2)
                if not windows:
                    raise RuntimeError(
                        f"{name}: no onscreen window; exit={process.poll()}; log={log}"
                    )
                time.sleep(0.5)
                if process.poll() is not None:
                    raise RuntimeError(f"{name} exited after opening its window")
                window = max(
                    windows,
                    key=lambda w: w["kCGWindowBounds"]["Width"]
                    * w["kCGWindowBounds"]["Height"],
                )
                image = output / (name + ".png")
                subprocess.run(
                    ["screencapture", "-x", "-l", str(window["kCGWindowNumber"]), str(image)],
                    check=True,
                )
                result = {
                    "demo": name,
                    "pid": process.pid,
                    "onscreen": True,
                    "window": window,
                    "binary_mtime": datetime.datetime.fromtimestamp(
                        binary.stat().st_mtime, datetime.timezone.utc
                    ).isoformat(),
                    "screenshot": str(image.relative_to(root)),
                    "log": str(log.relative_to(root)),
                }
                results.append(result)
                print(f"{name}: onscreen window captured", flush=True)
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()

report = {
    "checked_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "platform": "macOS",
    "demos": results,
    "note": "Real process and onscreen native-window checks plus screenshots; not an interaction or IME test.",
}
(output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
