"""Real process-group lifecycle tests for check-downstream._run."""
import importlib.util
import os
import signal
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "check-downstream.py"
SPEC = importlib.util.spec_from_file_location("check_downstream_processes", SCRIPT)
check = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = check
SPEC.loader.exec_module(check)


def _pid_running(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    proc_stat = Path(f"/proc/{pid}/stat")
    if proc_stat.exists():
        try:
            return proc_stat.read_text().split(")", 1)[1].split()[0] != "Z"
        except (OSError, IndexError):
            return False
    return True


def _wait_gone(pids, timeout=3):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not any(_pid_running(pid) for pid in pids):
            return
        time.sleep(0.025)
    still_running = [pid for pid in pids if _pid_running(pid)]
    if still_running:
        raise AssertionError(f"workers still running: {still_running}")


CHILD = r'''
import os, signal, subprocess, sys, time
record, mode = sys.argv[1:]
grandchild = subprocess.Popen([sys.executable, "-c", "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)"])
with open(record, "w", encoding="utf-8") as f:
    f.write(f"{os.getpid()} {grandchild.pid} {os.getpgid(0)}\n")
    f.flush()
if mode == "exit":
    time.sleep(0.15)
    raise SystemExit(0)
grandchild.wait()
'''

PARENT = r'''
import subprocess, sys, time
subprocess.Popen([sys.executable, "-c", %r, sys.argv[1], sys.argv[2]])
if sys.argv[2] == "exit":
    time.sleep(0.4)
else:
    time.sleep(60)
''' % CHILD


class DownstreamProcessTests(unittest.TestCase):
    def setUp(self):
        if os.name != "posix":
            self.skipTest("real process-group tests require POSIX")
        self.temp = tempfile.TemporaryDirectory(prefix="rs-spi-process-test-")
        self.root = Path(self.temp.name)
        self.record = self.root / "workers.txt"
        self.log = self.root / "command.log"
        self.pids = []

    def tearDown(self):
        # Ensure a failed assertion cannot leave workers behind.
        for pid in self.pids:
            try:
                os.kill(pid, signal.SIGKILL)
            except (ProcessLookupError, PermissionError):
                pass
        self.temp.cleanup()

    def _command(self, mode):
        return [sys.executable, "-c", PARENT, str(self.record), mode]

    def _recorded(self, timeout=2):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline and not self.record.exists():
            time.sleep(0.01)
        self.assertTrue(self.record.exists(), "descendant did not complete readiness handshake")
        child, grandchild, pgid = map(int, self.record.read_text().split())
        self.pids.extend((child, grandchild))
        return child, grandchild, pgid

    def test_success_and_nonzero_exit_capture_output_and_cleanup(self):
        success = check._run([sys.executable, "-c", "print('out'); print('err', file=__import__('sys').stderr)"], self.root, os.environ.copy(), 2, self.log)
        self.assertEqual(success.returncode, 0)
        self.assertEqual(success.cleanup_status, "complete")
        self.assertIn("out", success.stdout)
        self.assertIn("err", success.log)
        self.assertEqual(self.log.read_text(), success.log)
        failure = check._run([sys.executable, "-c", "raise SystemExit(7)"], self.root, os.environ.copy(), 2, self.log)
        self.assertEqual(failure.returncode, 7)
        self.assertEqual(failure.cleanup_status, "complete")

    def test_parent_exit_does_not_leave_descendant_running(self):
        result = check._run(self._command("exit"), self.root, os.environ.copy(), 2, self.log)
        _, grandchild, pgid = self._recorded()
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.process_group, pgid)
        self.assertEqual(result.cleanup_status, "complete")
        _wait_gone(self.pids)

    def test_timeout_kills_term_ignoring_grandchild_and_does_not_wait_on_pipe(self):
        started = time.monotonic()
        result = check._run(self._command("wait"), self.root, os.environ.copy(), 0.35, self.log)
        _, grandchild, _ = self._recorded()
        self.assertTrue(result.timed_out)
        self.assertIsNone(result.returncode)
        self.assertEqual(result.cleanup_status, "complete")
        self.assertLess(time.monotonic() - started, 4)
        _wait_gone(self.pids)
        self.assertFalse(_pid_running(grandchild))

    def test_exception_after_spawn_still_cleans_process_group(self):
        original_wait = check.subprocess.Popen.wait
        thrown = {"done": False}

        def interrupted(proc, *args, **kwargs):
            if not thrown["done"]:
                thrown["done"] = True
                # Let the child script establish the process tree before interruption.
                deadline = time.monotonic() + 1
                while not self.record.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                raise KeyboardInterrupt
            return original_wait(proc, *args, **kwargs)

        with mock.patch.object(check.subprocess.Popen, "wait", interrupted):
            with self.assertRaises(KeyboardInterrupt):
                check._run(self._command("wait"), self.root, os.environ.copy(), 2, self.log)
        self.assertTrue(self.log.exists(), "exception path should retain the command log")
        self._recorded()
        _wait_gone(self.pids)

    def test_cleanup_failure_is_recorded_and_cannot_pass(self):
        with mock.patch.object(check, "_cleanup_process_group", return_value=("failed", "permission denied")):
            result = check._run([sys.executable, "-c", "pass"], self.root, os.environ.copy(), 2, self.log)
        self.assertEqual(result.cleanup_status, "failed")
        self.assertIn("permission denied", result.cleanup_error)
        with self.assertRaisesRegex(ValueError, "process cleanup failed"):
            check._require_command_cleanup(result, "test fixture")

    def test_unsupported_platform_fails_before_commands(self):
        with mock.patch.object(check, "_require_posix_process_groups", side_effect=RuntimeError("unsupported")):
            with self.assertRaisesRegex(RuntimeError, "unsupported"):
                check._run([sys.executable, "-c", "pass"], self.root, os.environ.copy(), 2, self.log)

    def test_missing_process_listing_capability_fails_early(self):
        with mock.patch.object(check.Path, "is_dir", return_value=False), mock.patch.object(
                check.shutil, "which", return_value=None):
            with self.assertRaisesRegex(RuntimeError, r"requires /proc or a working POSIX ps"):
                check._require_posix_process_groups()

    def test_ps_capability_is_probed_before_workload(self):
        completed = check.subprocess.CompletedProcess(
            ["ps"], 0, stdout="12 S\n", stderr="")
        with mock.patch.object(check.Path, "is_dir", return_value=False), mock.patch.object(
                check.shutil, "which", return_value="/usr/bin/ps"), mock.patch.object(
                check.subprocess, "run", return_value=completed) as run_ps:
            check._require_posix_process_groups()
            run_ps.assert_called_once_with(["/usr/bin/ps", "-e", "-o", "pgid=,state="],
                                           capture_output=True, text=True, timeout=5, check=True)
        with mock.patch.object(check.Path, "is_dir", return_value=False), mock.patch.object(
                check.shutil, "which", return_value="/usr/bin/ps"), mock.patch.object(
                check.subprocess, "run",
                side_effect=check.subprocess.CalledProcessError(1, ["ps"])):
            with self.assertRaisesRegex(RuntimeError, r"requires /proc or a working POSIX ps"):
                check._require_posix_process_groups()

    def test_ps_fallback_distinguishes_live_members_from_zombies(self):
        pgid = 4567
        ps = mock.Mock(return_value="/usr/bin/ps")
        with mock.patch.object(check.shutil, "which", ps), mock.patch.object(
                check.subprocess, "run",
                return_value=check.subprocess.CompletedProcess(
                    ["ps"], 0, stdout="12 S\n4567 Z\n4567 Z+\n", stderr="")) as run_ps:
            self.assertFalse(check._group_has_live_members_from_ps(pgid))
            run_ps.assert_called_once_with(["/usr/bin/ps", "-e", "-o", "pgid=,state="],
                                           capture_output=True, text=True, timeout=5, check=True)
        with mock.patch.object(check.shutil, "which", ps), mock.patch.object(
                check.subprocess, "run",
                return_value=check.subprocess.CompletedProcess(
                    ["ps"], 0, stdout="4567 S\n4567 Z\n", stderr="")):
            self.assertTrue(check._group_has_live_members_from_ps(pgid))


if __name__ == "__main__":
    unittest.main()
