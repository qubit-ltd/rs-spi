import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import test_downstream_checks as downstream_tests

check = downstream_tests.check


class CargoDriverTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="cargo driver path ")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.native = self.base / "toolchain with spaces" / "bin" / "cargo"
        self.native.parent.mkdir(parents=True)
        self.native.write_bytes(b"native cargo binary fixture")
        self.native.chmod(0o755)
        self.env = {"PATH": "/existing/wrapper:/usr/bin"}

    def mock_run(self, version="cargo 1.94.0 (fixture)", copied_version=None):
        responses = [SimpleNamespace(stdout=str(self.native) + "\n")]
        responses.append(SimpleNamespace(stdout=version + "\n"))
        responses.append(SimpleNamespace(stdout=(copied_version or version) + "\n"))
        return mock.patch.object(check.subprocess, "run", side_effect=responses)

    def test_copies_native_driver_with_version_digest_and_executable_mode(self):
        with self.mock_run() as run:
            driver = check.prepare_cargo_driver("1.94.0", self.base / "destination with spaces", self.env)
        self.assertEqual(driver.name, "rs-spi-cargo-driver")
        self.assertEqual(driver.read_bytes(), self.native.read_bytes())
        self.assertTrue(driver.stat().st_mode & 0o111)
        self.assertEqual(run.call_args_list[0].args[0], ["rustup", "which", "--toolchain", "1.94.0", "cargo"])
        info = json.loads(driver.with_suffix(".json").read_text())
        digest = hashlib.sha256(self.native.read_bytes()).hexdigest()
        self.assertEqual(info["original_sha256"], digest)
        self.assertEqual(info["driver_sha256"], digest)
        self.assertIn("toolchain with spaces", info["original_path"])

    def test_rejects_rustup_proxy_symlink(self):
        proxy = self.base / "cargo-proxy"
        proxy.symlink_to(self.native)
        with mock.patch.object(check.subprocess, "run", return_value=SimpleNamespace(stdout=str(proxy))):
            with self.assertRaisesRegex(ValueError, "native Cargo file"):
                check.prepare_cargo_driver("1.94.0", self.base, self.env)

    def test_rejects_wrong_toolchain_version(self):
        with self.mock_run(version="cargo 1.93.0 (fixture)"):
            with self.assertRaisesRegex(ValueError, "version mismatch"):
                check.prepare_cargo_driver("1.94.0", self.base, self.env)

    def test_reports_copy_or_driver_load_failure(self):
        with mock.patch.object(check.subprocess, "run", side_effect=[
                SimpleNamespace(stdout=str(self.native)),
                SimpleNamespace(stdout="cargo 1.94.0 (fixture)"),
                OSError("dynamic loader rejected copied driver")]):
            with self.assertRaisesRegex(ValueError, "cannot prepare native Cargo driver"):
                check.prepare_cargo_driver("1.94.0", self.base, self.env)

    def test_reports_copy_failure(self):
        with self.mock_run() as run, mock.patch.object(check.shutil, "copy2", side_effect=OSError("copy denied")):
            with self.assertRaisesRegex(ValueError, "cannot prepare native Cargo driver"):
                check.prepare_cargo_driver("1.94.0", self.base, self.env)
        self.assertEqual(run.call_count, 2)

    def test_rejects_invalid_toolchain_name(self):
        with self.assertRaisesRegex(ValueError, "invalid Cargo toolchain"):
            check.prepare_cargo_driver("-V", self.base, self.env)

    def test_rustup_failure_is_reported_without_fallback(self):
        with mock.patch.object(check.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "rustup")):
            with self.assertRaisesRegex(ValueError, "cannot resolve native Cargo"):
                check.prepare_cargo_driver("1.94.0", self.base, self.env)

    def make_workspace(self):
        workspace = self.base / "workspace"
        for name in ("rs-spi", *check.CONSUMERS):
            root = workspace / name
            root.mkdir(parents=True)
            (root / "Cargo.toml").write_text(
                f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n')
            (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel="1.94.0"\n')
        return workspace

    def test_toolchain_mismatch_fails_before_any_command(self):
        workspace = self.make_workspace()
        (workspace / "rs-magika" / "rust-toolchain.toml").write_text('[toolchain]\nchannel="1.93.0"\n')
        calls = []
        def runner(*args):
            calls.append(args)
            raise AssertionError("consumer command must not run")
        report = self.base / "mismatch.json"
        self.assertEqual(check.run(workspace, workspace / "rs-spi", report, runner), 1)
        document = json.loads(report.read_text())
        self.assertFalse(document["success"])
        self.assertTrue(any("unsupported toolchain" in error for error in document["errors"]))
        self.assertEqual(calls, [])

    def test_probe_failure_stops_before_consumers_start(self):
        workspace = self.make_workspace()
        calls = []
        def runner(argv, cwd, env, timeout, log_path):
            calls.append(list(argv))
            Path(log_path).parent.mkdir(parents=True, exist_ok=True)
            Path(log_path).write_text("probe entry failed")
            return check.CommandResult(args=list(argv), cwd=str(cwd), returncode=1,
                timed_out=False, elapsed=0.01, log="probe entry failed", log_path=str(log_path))
        report = self.base / "probe-failure.json"
        self.assertEqual(check.run(workspace, workspace / "rs-spi", report, runner), 1)
        document = json.loads(report.read_text())
        self.assertFalse(document["success"])
        self.assertTrue(any("capability probe failed" in error for error in document["errors"]))
        self.assertEqual(len(calls), 1)
        self.assertIn("cargo-entry-probe", " ".join(calls[0]))


if __name__ == "__main__":
    unittest.main()
