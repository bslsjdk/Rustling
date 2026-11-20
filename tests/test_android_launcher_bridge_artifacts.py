import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "check_android_launcher_bridge", ROOT / "tools/check_android_launcher_bridge.py"
)
check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check)


class LauncherArtifacts(unittest.TestCase):
    def test_empty_abi_is_rejected_before_searching(self):
        with self.assertRaisesRegex(AssertionError, "invalid Android ABI"):
            check.check(Path("unused"), "")


    def test_apk_with_the_bridge_is_accepted(self):
        with tempfile.TemporaryDirectory() as folder:
            out = Path(folder)
            classes = out / "classes"
            (classes / "io/github/juanluci/starling").mkdir(parents=True)
            # The CI uses javap against the compiled facade. This test verifies the APK selection
            # and ABI independently of the host Kotlin installation.
            (out / "classes.jar").write_bytes(b"jar")
            with zipfile.ZipFile(out / "starling-preview-0.8.0-arm64-v8a.apk", "w") as apk:
                apk.writestr("lib/arm64-v8a/librustling_launcher.so", b"arm64")
            with mock.patch.object(subprocess, "run", return_value=mock.Mock(
                stdout="private static final native java.lang.String dispatch(java.lang.String);"
            )):
                check.check(out, "arm64-v8a")

    def test_missing_launcher_library_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            out = Path(folder)
            (out / "classes.jar").write_bytes(b"jar")
            with zipfile.ZipFile(out / "starling-preview-0.8.0-arm64-v8a.apk", "w") as apk:
                apk.writestr("lib/x86_64/librustling_launcher.so", b"x86")
            with mock.patch.object(subprocess, "run", return_value=mock.Mock(
                stdout="private static final native java.lang.String dispatch(java.lang.String);"
            )):
                with self.assertRaises(AssertionError):
                    check.check(out, "arm64-v8a")


if __name__ == "__main__":
    unittest.main()
