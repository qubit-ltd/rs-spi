"""Contract tests for the local project CI hook."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
HOOK = PROJECT_ROOT / '.infra' / 'project-ci-check.local.sh'
EXPECTED = [
    ['-m', 'unittest', 'discover', '-s', 'scripts/tests', '-p', 'test_*.py'],
    ['scripts/check-documentation.py'],
    ['scripts/check-package.py'],
]


class ProjectCiHookTests(unittest.TestCase):
    def invoke(self, allow_dirty=None, fail_at=None, caller_cwd=None):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fake_bin = root / 'bin'
            fake_bin.mkdir()
            log_path = root / 'calls.jsonl'
            fake_python = fake_bin / 'python3'
            fake_python.write_text(
                '#!/usr/bin/python3\n'
                'import json, os, sys\n'
                'with open(os.environ["HOOK_LOG"], "a", encoding="utf-8") as log:\n'
                '    log.write(json.dumps({"argv": sys.argv[1:], "cwd": os.getcwd()}) + "\\n")\n'
                'sys.exit(17 if len(open(os.environ["HOOK_LOG"], encoding="utf-8").readlines()) == int(os.environ.get("HOOK_FAIL_AT", "0")) else 0)\n',
                encoding='utf-8',
            )
            fake_python.chmod(0o755)
            env = os.environ.copy()
            env['PATH'] = f'{fake_bin}{os.pathsep}{env.get("PATH", "")}'
            env['HOOK_LOG'] = str(log_path)
            if allow_dirty is not None:
                env['SPI_PACKAGE_ALLOW_DIRTY'] = allow_dirty
            else:
                env.pop('SPI_PACKAGE_ALLOW_DIRTY', None)
            env['HOOK_FAIL_AT'] = str(fail_at or 0)
            result = subprocess.run(
                ['bash', str(HOOK)], cwd=caller_cwd or PROJECT_ROOT, env=env,
                text=True, capture_output=True, check=False,
            )
            records = [json.loads(line) for line in log_path.read_text(encoding='utf-8').splitlines()] if log_path.exists() else []
            return result, records

    def assert_calls(self, records, expected, count=None):
        self.assertEqual(count if count is not None else len(expected), len(records))
        for record, argv in zip(records, expected):
            self.assertEqual(argv, record['argv'])
            self.assertEqual(str(PROJECT_ROOT), record['cwd'])

    def test_default_empty_and_zero_run_all_commands_from_project_root(self):
        for value in (None, '', '0'):
            with self.subTest(value=value):
                result, records = self.invoke(allow_dirty=value)
                self.assertEqual(0, result.returncode, result.stderr)
                self.assert_calls(records, EXPECTED)

    def test_one_adds_allow_dirty_only_to_package_check(self):
        result, records = self.invoke(allow_dirty='1')
        self.assertEqual(0, result.returncode, result.stderr)
        expected = [*EXPECTED[:2], [*EXPECTED[2], '--allow-dirty']]
        self.assert_calls(records, expected)

    def test_invalid_value_exits_two_without_running_python(self):
        result, records = self.invoke(allow_dirty='2')
        self.assertEqual(2, result.returncode)
        self.assertEqual([], records)

    def test_subcommand_failure_stops_immediately_and_preserves_status(self):
        for failed_call in (1, 2, 3):
            with self.subTest(failed_call=failed_call):
                result, records = self.invoke(fail_at=failed_call)
                self.assertEqual(17, result.returncode)
                self.assert_calls(records, EXPECTED, count=failed_call)

    def test_external_caller_directory_does_not_change_commands_or_cwd(self):
        with tempfile.TemporaryDirectory() as external_cwd:
            result, records = self.invoke(caller_cwd=external_cwd)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assert_calls(records, EXPECTED)
