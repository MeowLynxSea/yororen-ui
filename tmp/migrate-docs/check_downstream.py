#!/usr/bin/env python3
"""Resolve a fresh workspace and compile a consumer without patches or a lock."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


root = Path.cwd()
stage = Path(tempfile.mkdtemp(prefix="yororen-gpui-migration-"))
workspace = stage / "workspace"
workspace.mkdir()
for name in ["Cargo.toml", "rust-toolchain.toml", "README.md", "LICENSE"]:
    if (root / name).is_file():
        shutil.copy2(root / name, workspace / name)
shutil.copytree(
    root / "crates",
    workspace / "crates",
    ignore=shutil.ignore_patterns("target", "Cargo.lock", "fuzz", ".DS_Store"),
)
environment = os.environ.copy()
environment["CARGO_TARGET_DIR"] = str(root / "target")
environment["RUSTUP_TOOLCHAIN"] = "1.95.0"


def run(arguments, cwd):
    subprocess.run(arguments, cwd=cwd, env=environment, check=True)


run(["cargo", "generate-lockfile"], workspace)
fresh = tomllib.loads((workspace / "Cargo.lock").read_text())["package"]
original = tomllib.loads((root / "Cargo.lock").read_text())["package"]


def external(packages):
    return {
        (p["name"], p["version"], p["source"], p["checksum"])
        for p in packages
        if "source" in p
    }


assert external(fresh) == external(original), "Fresh resolution changed external packages"
assert not list(workspace.glob(".cargo/config*")), "Unexpected dependency configuration"

consumer = stage / "consumer"
(consumer / "src").mkdir(parents=True)
library = workspace / "crates/yororen-ui"
(consumer / "Cargo.toml").write_text(
    f"""[package]
name = "yororen-migration-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "1.95"

[workspace]

[dependencies]
yororen_ui = {{ path = {json.dumps(str(library))}, features = ["winui"] }}
gpui = {{ package = "gpui-ce", version = "0.3.0" }}
gpui-ce-platform = {{ package = "gpui_ce_platform", version = "0.3.0", features = ["font-kit", "wayland", "x11"] }}
"""
)
(consumer / "src/main.rs").write_text(
    """use gpui::{App, Context, IntoElement, Render, Window, WindowOptions, prelude::*};
use yororen_ui::{assets::UiAsset, headless::button::button};

struct Example;

impl Render for Example {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        button("hello", cx).caption("Hello").render(cx)
    }
}

fn main() {
    gpui_ce_platform::application().with_assets(UiAsset).run(|cx: &mut App| {
        yororen_ui::renderer::install(cx, cx.window_appearance());
        yororen_ui::headless::text_input::init(cx);
        yororen_ui::locale_en::install(cx);
        let _ = cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Example));
    });
}
"""
)
assert not (consumer / "Cargo.lock").exists()
run(["cargo", "check"], consumer)
metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=consumer,
        env=environment,
    )
)
gpui = [p for p in metadata["packages"] if p["name"] == "gpui-ce"]
assert len(gpui) == 1 and gpui[0]["version"] == "0.3.0"
for package in metadata["packages"]:
    if package["name"].startswith("yororen_ui"):
        assert package["version"] == "0.4.0"
    elif package["name"] != "yororen-migration-consumer":
        assert package["source"] == "registry+https://github.com/rust-lang/crates.io-index"
report = {
    "stage": str(stage),
    "fresh_workspace_external_versions_match_audited_lock": True,
    "external_package_versions": len(external(fresh)),
    "consumer_check": "passed",
    "gpui_versions": [p["version"] for p in gpui],
    "patches": False,
    "git_dependencies": False,
    "note": "Only unpublished Yororen 0.4 source uses local paths; all upstream dependencies resolve normally from crates.io.",
}
(root / "tmp/migrate-docs/downstream-check.json").write_text(
    json.dumps(report, indent=2) + "\n"
)
print(json.dumps(report), flush=True)
