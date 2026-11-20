"""APK ABI entries are consumed by Android's native-library installer."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("starling_apk_abis", ROOT / "packaging/android/build_apk.py")
apk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(apk)


class ApkAbis(unittest.TestCase):
    def test_native_libraries_are_packaged_only_under_the_selected_abi(self):
        for abi in ("arm64-v8a", "armeabi-v7a", "x86", "x86_64"):
            with self.subTest(abi=abi), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                base, output = root / "base.apk", root / "output.apk"
                with zipfile.ZipFile(base, "w") as archive:
                    archive.writestr("AndroidManifest.xml", b"manifest")
                library, dex = root / "libhorizon_android.so", root / "classes.dex"
                launcher = root / "librustling_launcher.so"
                library.write_bytes(b"native library")
                launcher.write_bytes(b"launcher library")
                dex.write_bytes(b"dex bytecode")
                apk.add_code(base, {library.name: library, launcher.name: launcher}, dex, output, abi)
                with zipfile.ZipFile(output) as archive:
                    self.assertEqual(set(archive.namelist()), {
                        "AndroidManifest.xml", "classes.dex", f"lib/{abi}/libhorizon_android.so",
                        f"lib/{abi}/librustling_launcher.so",
                    })
                    self.assertEqual(archive.read(f"lib/{abi}/libhorizon_android.so"), b"native library")
                    self.assertEqual(archive.read(f"lib/{abi}/librustling_launcher.so"), b"launcher library")
                    self.assertEqual(archive.read("AndroidManifest.xml"), b"manifest")

    def test_cargo_builds_the_game_and_launcher_for_the_same_abi(self):
        for abi in apk.TARGETS:
            with self.subTest(abi=abi), mock.patch.object(apk, "run") as run:
                apk.build_library("quick", Path("/ndk"), Path("/sdk"), abi)
            command = run.call_args.args
            self.assertEqual(command[:4], ("cargo", "ndk", "-t", abi))
            self.assertIn("horizon-android", command)
            self.assertIn("starling-launcher-core", command)


    def test_library_crate_name_matches_the_packaged_library(self):
        manifest = (apk.ROOT / "crates/starling-launcher-core/Cargo.toml").read_text()
        self.assertIn(f'name = "{apk.LAUNCHER_LIBRARY}"', manifest)
        self.assertIn('crate-type = ["rlib", "cdylib"]', manifest)


class KotlinBuild(unittest.TestCase):
    def test_kotlin_artifacts_are_pinned_and_facade_targets_jvm_8(self):
        self.assertEqual(apk.KOTLIN_VERSION, "2.4.21")
        self.assertEqual(apk.COROUTINES_VERSION, "1.10.2")
        facade = (apk.HERE / "kotlin/io/github/juanluci/starling/RustlingCore.kt").read_text()
        self.assertIn('System.loadLibrary("rustling_launcher")', facade)
        self.assertIn('private external fun dispatch(request: String): String', facade)
        symbol = (apk.ROOT / "crates/starling-launcher-core/src/jni.rs").read_text()
        self.assertIn("Java_io_github_juanluci_starling_RustlingCore_dispatch", symbol)


if __name__ == "__main__":
    unittest.main()
