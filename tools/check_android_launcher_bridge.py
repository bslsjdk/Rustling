#!/usr/bin/env python3
"""Check the APK's launcher library and the Kotlin facade's native dispatch method."""
import argparse
from pathlib import Path
import subprocess
import zipfile


def check(out: Path, abi: str) -> None:
    assert abi in {"arm64-v8a", "armeabi-v7a", "x86", "x86_64"}, f"invalid Android ABI: {abi!r}"
    signature = "private static final native java.lang.String dispatch(java.lang.String);"
    javap = subprocess.run(
        ["javap", "-classpath", out / "classes.jar", "-private",
         "io.github.juanluci.starling.RustlingCore"],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout
    assert signature in javap, f"Kotlin's native dispatch method is missing: {javap}"
    apks = sorted(out.glob(f"starling-*-{abi}.apk"))
    assert len(apks) == 1, f"expected one {abi} APK, found {[p.name for p in apks]}"
    with zipfile.ZipFile(apks[0]) as archive:
        names = archive.namelist()
        assert f"lib/{abi}/librustling_launcher.so" in names, "launcher library missing"
        if abi == "arm64-v8a":
            assert not any(name.startswith(("lib/x86/", "lib/x86_64/")) for name in names)
    print(f"{apks[0]}: launcher bridge and {abi} ABI verified")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("out", type=Path)
    parser.add_argument("abi")
    args = parser.parse_args()
    check(args.out, args.abi)
