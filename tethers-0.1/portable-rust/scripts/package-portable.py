"""Cross-platform deterministic bundle builder for an already-built binary."""
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", choices=("windows-x64", "linux-x64-musl"), required=True)
    parser.add_argument("--binary", required=True)
    args = parser.parse_args()
    root = Path(__file__).parents[1]
    version = (root / "VERSION").read_text(encoding="utf-8").strip()
    dist = root / "dist"
    stage = dist / f"tethers-portable-{version}-{args.target}"
    archive = dist / f"tethers-portable-{version}-{args.target}.zip"
    if stage.exists(): shutil.rmtree(stage)
    if archive.exists(): archive.unlink()
    for directory in ("bin", "policies", "registry", "plugs", "docs", "tests", "scripts", "schemas", "examples",
                      "wrappers/rust", "wrappers/go", "wrappers/typescript", "wrappers/python"):
        (stage / directory).mkdir(parents=True, exist_ok=True)
    binary = Path(args.binary)
    if not binary.is_file(): raise SystemExit(f"binary not found: {binary}")
    shutil.copy2(binary, stage / "bin" / ("tethers.exe" if args.target == "windows-x64" else "tethers"))
    for directory in ("policies", "registry", "plugs", "docs", "tests", "schemas", "examples"):
        for source in (root / directory).iterdir():
            if source.is_file(): shutil.copy2(source, stage / directory / source.name)
    for source in (root / "wrappers").rglob("*"):
        if source.is_file():
            destination = stage / source.relative_to(root)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
    for name in ("run_parity.py", "benchmark.py"):
        shutil.copy2(root / "scripts" / name, stage / "scripts" / name)
    for name in ("README.md", "RELEASE.md", "QUICKSTART.md", "AI-INTEGRATION.md", "VERSION"):
        shutil.copy2(root / name, stage / name)
    executable = stage / "bin" / ("tethers.exe" if args.target == "windows-x64" else "tethers")
    (stage / "SHA256SUMS").write_text(f"{hashlib.sha256(executable.read_bytes()).hexdigest().upper()}  {executable.relative_to(stage).as_posix()}\n", encoding="utf-8")
    subprocess.run([sys.executable, str(root / "scripts" / "deterministic_zip.py"), str(stage), str(archive)], check=True)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest().upper()
    archive.with_suffix(archive.suffix + ".sha256").write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
    print(json.dumps({"bundle": str(archive), "sha256": digest}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
