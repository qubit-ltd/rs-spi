import importlib.util
import json
import sys
import subprocess
import tempfile
import unittest
from unittest import mock
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "check-downstream.py"
SPEC = importlib.util.spec_from_file_location("downstream", SCRIPT)
check = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = check
SPEC.loader.exec_module(check)


class DownstreamTests(unittest.TestCase):
    def test_origin_requires_the_unique_reachable_snapshot_spi(self):
        spi = Path("/snapshot/rs-spi/Cargo.toml")
        good = {"packages": [{"id": "spi", "name": "qubit-spi", "source": None,
                              "manifest_path": str(spi)}],
                "resolve": {"root": "app", "nodes": [
                    {"id": "app", "dependencies": ["spi"]},
                    {"id": "spi", "dependencies": []}]}}
        check.assert_spi_origin(good, spi)
        for metadata in (
            {**good, "packages": [{**good["packages"][0], "source": "registry+x"}]},
            {**good, "resolve": {"root": "app", "nodes": [
                {"id": "app", "dependencies": ["spi", "spi2"]},
                {"id": "spi", "dependencies": []},
                {"id": "spi2", "dependencies": []}]},
             "packages": good["packages"] + [{"id": "spi2", "name": "qubit-spi",
                   "source": None, "manifest_path": str(spi)}]},
            {**good, "resolve": {"root": "app", "nodes": [{"id": "app", "dependencies": []}] }},
        ):
            with self.subTest(metadata=metadata), self.assertRaises(ValueError):
                check.assert_spi_origin(metadata, spi)

    def test_dependency_closure_covers_nested_manifests_and_rejects_escape(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            workspace = base / "workspace"
            workspace.mkdir()
            consumer = workspace / "rs-mime"
            (consumer / "tests" / "fixture").mkdir(parents=True)
            dep = workspace / "rs-common"
            dep.mkdir()
            spi = workspace / "rs-spi"
            spi.mkdir()
            (consumer / "Cargo.toml").write_text('''[package]\nname="consumer"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nx={path="../rs-common"}\n[dev-dependencies]\ny={path="../rs-common"}\n''')
            with (consumer / "Cargo.toml").open("a") as f:
                f.write('''\n[build-dependencies]\nb={path="../rs-common"}\n[target.'cfg(unix)'.dependencies]\nt={path="../rs-common"}\n[workspace]\nmembers=["tests/fixture"]\n[workspace.dependencies]\nw={path="../rs-common"}\n[patch.crates-io]\npatched={path="../rs-common"}\n[replace]\n"old:0.1.0"={path="../rs-common"}\n[lib]\npath="not-a-dependency.rs"\n''')
            (consumer / "tests/fixture/Cargo.toml").write_text('''[package]\nname="fixture"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nspi={path="../../../rs-spi"}\n''')
            (dep / "Cargo.toml").write_text('''[package]\nname="dep"\nversion="0.1.0"\nedition="2021"\n''')
            (spi / "Cargo.toml").write_text('''[package]\nname="qubit-spi"\nversion="0.1.0"\nedition="2021"\n''')
            roots = check.discover_inputs(workspace, [consumer], spi)
            self.assertEqual(set(roots), {consumer.resolve(), (consumer / "tests/fixture").resolve(), dep.resolve(), spi.resolve()})
            outside = base / "outside"
            outside.mkdir()
            (consumer / "Cargo.toml").write_text((consumer / "Cargo.toml").read_text() +
                                                  '\n[build-dependencies]\nz={path="../../outside"}\n')
            with self.assertRaises(ValueError):
                check.discover_inputs(workspace, [consumer], spi)

    def test_cargo_1_1_multiline_inline_dependency_table_is_discovered(self):
        with tempfile.TemporaryDirectory() as temp:
            workspace = Path(temp) / "workspace"
            consumer = workspace / "rs-mime"
            fixture = workspace / "rs-redact/tests/fixtures/crates/renamed_dependency"
            fixture_support = workspace / "rs-redact/tests"
            spi = workspace / "rs-spi"
            for root in (consumer, fixture, fixture_support, spi):
                root.mkdir(parents=True, exist_ok=True)
            (consumer / "Cargo.toml").write_text('''[package]
name = "consumer"
version = "0.1.0"
edition = "2021"
[dependencies]
renamed = { package = "qubit-redact", path = "../rs-redact/tests/fixtures/crates/renamed_dependency" }
''')
            (fixture / "Cargo.toml").write_text('''[package]
name = "qubit-redact"
version = "0.1.0"
edition = "2021"
[dependencies]
redaction-runtime = {
    package = "qubit-redact-support", # comment containing } and \"quotes\"
    path = "../../..",
    default-features = false,
}
[package.metadata.lexer]
basic = "brace } and # marker"
literal = 'brace } and # marker'
multiline = """line one
line two } # still a string"""
''')
            (fixture_support / "Cargo.toml").write_text('''[package]
name = "qubit-redact-support"
version = "0.1.0"
edition = "2021"
''')
            (spi / "Cargo.toml").write_text('''[package]
name = "qubit-spi"
version = "0.1.0"
edition = "2021"
''')
            inputs = check.discover_inputs(workspace, [consumer], spi)
            self.assertIn(fixture.resolve(), inputs)
            self.assertIn(fixture_support.resolve(), inputs)

    def test_source_symlink_is_rejected_without_following(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "root"
            root.mkdir()
            outside = Path(temp) / "outside"
            outside.mkdir()
            (root / "top.txt").write_text("top")
            (root / "nested").mkdir()
            (root / "nested/child.txt").write_text("child")
            (root / "link").symlink_to(outside, target_is_directory=True)
            _, inventory = check._tree_hash(root)
            self.assertEqual({"link", "top.txt", "nested/child.txt"},
                             {entry["path"] for entry in inventory})
            with self.assertRaises(ValueError):
                check.copy_tree_safe(root, Path(temp) / "copy", Path(temp))

    def test_timeout_keeps_diagnostic_snapshot_and_preserves_dirty_source(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            workspace = base / "workspace"
            workspace.mkdir()
            for name in ("rs-spi", *check.CONSUMERS):
                root = workspace / name
                root.mkdir()
                (root / "Cargo.toml").write_text(
                    f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n')
            mime = workspace / "rs-mime"
            subprocess.run(["git", "init", "-q", str(mime)], check=True)
            subprocess.run(["git", "-C", str(mime), "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                            "add", "Cargo.toml"], check=True)
            subprocess.run(["git", "-C", str(mime), "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                            "commit", "-qm", "baseline"], check=True)
            dirty = workspace / "rs-mime" / "uncommitted.txt"
            dirty.write_text("preserve me")
            before = dirty.read_bytes()
            report = base / "failure.json"
            def timeout_runner(argv, cwd, env, timeout, log_path):
                Path(log_path).parent.mkdir(parents=True, exist_ok=True)
                Path(log_path).write_text("secret diagnostic that must stay out of report")
                return check.CommandResult(args=argv, cwd=str(cwd), returncode=None,
                    timed_out=True, elapsed=timeout, log="secret diagnostic that must stay out of report",
                    log_path=str(log_path))
            self.assertEqual(check.run(workspace, workspace / "rs-spi", report, timeout_runner), 1)
            document = json.loads(report.read_text())
            self.assertFalse(document["success"])
            self.assertTrue(document["inputs"][str((workspace / "rs-mime").resolve())]["dirty"])
            self.assertEqual(dirty.read_bytes(), before)
            self.assertNotIn("secret diagnostic", report.read_text())
            self.assertTrue(Path(document["temporary_directory"]).exists())
            timed = document["commands"][0]
            self.assertTrue(timed["timed_out"])
            self.assertIsNone(timed["returncode"])
            self.assertTrue(Path(timed["log_path"]).exists())

    def test_real_cargo_nested_fixture_uses_spi_snapshot(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            workspace = base / "workspace"
            spi = workspace / "rs-spi"
            spi.mkdir(parents=True)
            (spi / "Cargo.toml").write_text('[package]\nname="qubit-spi"\nversion="0.1.0"\nedition="2021"\n')
            (spi / "src").mkdir()
            (spi / "src/lib.rs").write_text("pub fn marker() {}\n")
            for name in check.CONSUMERS:
                root = workspace / name
                fixture = root / "tests/fixture"
                (root / "tests").mkdir(parents=True)
                fixture.mkdir(parents=True)
                (root / "Cargo.toml").write_text(
                    f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nqubit-spi="0.1.0"\n')
                (root / "src").mkdir()
                (root / "src/lib.rs").write_text("pub fn use_spi() { qubit_spi::marker(); }\n")
                (fixture / "Cargo.toml").write_text(
                    f'[package]\nname="{name}-fixture"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nqubit-spi="0.1.0"\n[features]\nfixture-feature=[]\n')
                (fixture / "src").mkdir()
                (fixture / "src/lib.rs").write_text("pub fn use_spi() { qubit_spi::marker(); }\n")
                (fixture / "build.rs").write_text('''
fn main() {
    println!("cargo:rustc-env=CARGO={}", std::env::var("RS_SPI_CARGO_WRAPPER").unwrap());
}
''')
                (fixture / "tests").mkdir()
                grandchild = fixture / "tests/grandchild"
                (grandchild / "src").mkdir(parents=True)
                (grandchild / "Cargo.toml").write_text(
                    f'[package]\nname="{name}-grandchild"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nqubit-spi="0.1.0"\n')
                (grandchild / "src/lib.rs").write_text("pub fn use_spi() { qubit_spi::marker(); }\n")
                (fixture / "tests/nested.rs").write_text('''
use std::{process::Command, env};
#[test]
fn spawned_cargo_resolves_snapshot_spi() {
    let manifest = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let grandchild = format!("{}/tests/grandchild/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    // Cargo resets runtime CARGO to its own executable; bind the fixture's
    // compile-time wrapper path back into the runtime variable before lookup.
    env::set_var("CARGO", env!("CARGO"));
    let runtime_cargo = env::var_os("CARGO").expect("CARGO must be set by the checker");
    assert!(runtime_cargo.to_string_lossy().contains("cargo-wrapper"), "unexpected CARGO: {:?}", runtime_cargo);
    for cargo in ["cargo".to_string(), env!("CARGO").to_string(), runtime_cargo.to_string_lossy().into_owned()] {
        let output = Command::new(cargo).args(["metadata", "--format-version", "1", "--manifest-path", &manifest, "--no-default-features", "--features", "fixture-feature"]).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
    let output = Command::new("cargo").args(["metadata", "--format-version", "1", "--manifest-path", &grandchild]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
''')
                (root / "tests/launch_fixture.rs").write_text(f'''
use std::process::Command;
#[test]
fn launch_nested_fixture() {{
    let manifest = format!("{{}}/tests/fixture/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let status = Command::new("cargo").args(["test", "--manifest-path", &manifest]).status().unwrap();
    assert!(status.success());
}}
''')
            report = base / "report.json"
            with mock.patch.dict("os.environ", {"CARGO": "/caller/preexisting/cargo"}):
                result = check.run(workspace, spi, report)
            document = json.loads(report.read_text())
            self.assertEqual(result, 0, document["errors"])
            for consumer in document["consumers"]:
                for mode in consumer["modes"]:
                    fixtures = mode["executed_fixtures"]
                    self.assertTrue(all(item["source"] is None for item in fixtures))
                    self.assertTrue(all(item["spi_manifest_path"] == str(
                        (Path(document["temporary_directory"]) / "snapshot/rs-spi/Cargo.toml").resolve())
                        for item in fixtures))
                    manifests = [item["manifest_path"] for item in fixtures]
                    self.assertEqual(len(set(manifests)), 2, (consumer["name"], mode["mode"], manifests))
                    self.assertTrue(any("/tests/grandchild/" in path for path in manifests))
                    fixture_manifest = str(Path(document["temporary_directory"]) / "snapshot" /
                                           consumer["name"] / "tests/fixture/Cargo.toml")
                    grandchild_manifest = str(Path(document["temporary_directory"]) / "snapshot" /
                                              consumer["name"] / "tests/fixture/tests/grandchild/Cargo.toml")
                    self.assertEqual(manifests.count(fixture_manifest), 4, manifests)
                    self.assertEqual(manifests.count(grandchild_manifest), 1, manifests)
            nested_commands = [c for c in document["commands"] if any(
                "/tests/fixture/" in arg or "/tests/grandchild/" in arg
                for arg in c["argv"])]
            self.assertTrue(nested_commands)
            for command in nested_commands:
                self.assertIn("argv", command)
                self.assertIn("returncode", command)
                self.assertIn("timed_out", command)
                self.assertIn("elapsed_seconds", command)
                self.assertTrue(Path(command["log_path"]).is_absolute())
            metadata_preflights = [c for c in nested_commands if c["argv"] and "metadata" in c["argv"]]
            self.assertTrue(metadata_preflights)
            explicit_fixture_preflights = [c for c in metadata_preflights if any(
                "/tests/fixture/Cargo.toml" in arg for arg in c["argv"])
                and "--no-default-features" in c["argv"]]
            # Across four consumers and two modes, each of three explicit
            # launches has one preflight and one Cargo record. Recursive wrapper
            # retries would increase this count.
            self.assertEqual(len(explicit_fixture_preflights), 48)
            self.assertTrue(all("--features" in c["argv"] and "fixture-feature" in c["argv"]
                                for c in explicit_fixture_preflights))

    def test_cli_contract_and_report_redacts_environment(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            workspace = base / "workspace with ' quote"
            workspace.mkdir()
            for name in ("rs-spi", "rs-fs-registry", "rs-mime", "rs-magika", "rs-event-bus"):
                (workspace / name).mkdir()
                (workspace / name / "Cargo.toml").write_text(
                    f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n')
            report = base / "report.json"
            temp_root = base / "snapshot parent ' quote"
            temp_root.mkdir()
            previous_tempdir = check.tempfile.tempdir
            check.tempfile.tempdir = str(temp_root)
            calls = []
            def runner(argv, cwd, env, timeout, log_path):
                calls.append(list(argv))
                if "metadata" in argv:
                    spi_path = argv[argv.index("--config") + 1].split("=", 1)[1].strip('"')
                    data = {"packages": [
                        {"id": "app", "name": "app", "manifest_path": str(Path(cwd) / "Cargo.toml"), "source": None},
                        {"id": "spi", "name": "qubit-spi", "manifest_path": str(Path(spi_path) / "Cargo.toml"), "source": None}],
                        "resolve": {"root": "app", "nodes": [{"id": "app", "dependencies": ["spi"]}, {"id": "spi", "dependencies": []}]}}
                    output = json.dumps(data)
                else:
                    output = "ok"
                Path(log_path).parent.mkdir(parents=True, exist_ok=True)
                Path(log_path).write_text(output)
                return check.CommandResult(args=argv, cwd=str(cwd), returncode=0,
                                           timed_out=False, elapsed=0.01, log=output,
                                           log_path=str(log_path))
            try:
                result = check.run(workspace, workspace / "rs-spi", report,
                                   command_runner=runner)
            finally:
                check.tempfile.tempdir = previous_tempdir
            self.assertEqual(result, 0)
            doc = json.loads(report.read_text())
            self.assertEqual(doc["version"], 1)
            self.assertTrue(doc["success"])
            self.assertNotIn("OPENAI_API_KEY", report.read_text())
            meta = [c for c in calls if "metadata" in c]
            tests = [c for c in calls if "test" in c]
            self.assertEqual(len(meta), 8)
            self.assertEqual(len(tests), 8)
            self.assertTrue(all("--config" in c for c in meta + tests))
            self.assertTrue(all("--locked" in c for c in tests))
            self.assertTrue(all("--all-features" in c for c in tests[1::2]))
            self.assertTrue(any(any("snapshot parent ' quote" in arg for arg in c) for c in meta + tests))
            self.assertFalse(Path(doc["temporary_directory"]).exists())
            self.assertEqual(doc["cleanup"]["status"], "removed")

    def test_run_rejects_input_tree_symlink_and_preserves_diagnostics_and_sources(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            workspace = base / "workspace"
            workspace.mkdir()
            roots = {}
            for name in ("rs-spi", *check.CONSUMERS):
                root = workspace / name
                root.mkdir()
                (root / "Cargo.toml").write_text(
                    f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n')
                roots[name] = root
            event_bus = roots["rs-event-bus"]
            subprocess.run(["git", "init", "-q", str(event_bus)], check=True)
            subprocess.run(["git", "-C", str(event_bus), "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                            "add", "Cargo.toml"], check=True)
            subprocess.run(["git", "-C", str(event_bus), "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                            "commit", "-qm", "baseline"], check=True)
            link = event_bus / "source-tree-link"
            link.symlink_to(roots["rs-mime"], target_is_directory=True)
            before_hashes = {name: check._tree_hash(root)[0] for name, root in roots.items()}
            before_target = link.resolve()
            before_status = subprocess.run(["git", "-C", str(event_bus), "status", "--porcelain"],
                text=True, capture_output=True, check=True).stdout
            report = base / "report.json"
            self.assertEqual(check.run(workspace, roots["rs-spi"], report), 1)
            document = json.loads(report.read_text())
            self.assertFalse(document["success"])
            self.assertTrue(any("source-tree symlink rejected" in error for error in document["errors"]))
            self.assertTrue(Path(document["temporary_directory"]).exists())
            self.assertEqual(link.resolve(), before_target)
            self.assertTrue(link.is_symlink())
            self.assertEqual(before_status, subprocess.run(["git", "-C", str(event_bus), "status", "--porcelain"],
                text=True, capture_output=True, check=True).stdout)
            self.assertEqual(before_hashes,
                {name: check._tree_hash(root)[0] for name, root in roots.items()})


if __name__ == "__main__":
    unittest.main()
