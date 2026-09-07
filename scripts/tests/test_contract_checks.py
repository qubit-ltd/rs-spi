"""Regression tests for executable documentation and release archives."""
import importlib.util
from pathlib import Path, PurePosixPath
import sys
import unittest


def load_script(name):
    """Load a project script without depending on the working directory."""
    spec = importlib.util.spec_from_file_location(name, Path(__file__).parents[1] / f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


class DocumentationTests(unittest.TestCase):
    """Reject silent omissions and preserve the code readers actually see."""

    @classmethod
    def setUpClass(cls):
        cls.check = load_script('check-documentation')

    def test_reject_unsafe_paths(self):
        for value in ('', '.', '../outside.rs', '/outside.rs', 'app/../../x', 'a\\b'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.check.validate_relative_file(value)

    def test_extract_original_text(self):
        source = '<!-- spi-example: quick-start; file: app/src/main.rs -->\n```rust\nfn main() {}\n```\n'
        blocks = self.check.extract_blocks(source, 'README.md')
        self.assertEqual(blocks[0].code, 'fn main() {}\n')
        self.assertEqual(blocks[0].file, PurePosixPath('app/src/main.rs'))
        self.assertEqual(blocks[0].first_code_line, 3)

    def test_reject_missing_marker_and_unclosed_fence(self):
        for source in ('```rust\nfn main() {}\n```\n', '```text\nunclosed\n',
                       '<!-- spi-example: a; file: app/src/main.rs -->\n```rust,ignore\nfn main() {}\n```\n'):
            with self.subTest(source=source), self.assertRaises(ValueError):
                self.check.extract_blocks(source, 'README.md')

    def test_reject_space_separated_rust_exemptions(self):
        for language in ('rust no_run', 'rust ignore', 'rust,no_run'):
            source = f'```{language}\nfn main() {{}}\n```\n'
            with self.subTest(language=language), self.assertRaises(ValueError):
                self.check.extract_blocks(source, 'README.md')

    def test_reject_missing_scenario(self):
        with self.assertRaises(ValueError):
            self.check.validate_blocks([], {'quick-start': ['app/src/main.rs']}, 'README.md')

    def test_reject_duplicate_and_unknown_file(self):
        block = self.check.ExampleBlock('quick-start', PurePosixPath('app/src/main.rs'), 'fn main() {}', 'README.md', 3)
        for blocks, expected in (([block, block], {'quick-start': ['app/src/main.rs']}),
                                 ([block], {'quick-start': ['app/src/lib.rs']})):
            with self.assertRaises(ValueError):
                self.check.validate_blocks(blocks, expected, 'README.md')



class ArchiveTests(unittest.TestCase):
    """Archives must be self-contained regular files below the crate root."""

    def test_reject_unsafe_archive_members(self):
        import tarfile
        check = load_script('check-package')
        for name, kind in (('../escape', tarfile.REGTYPE), ('/escape', tarfile.REGTYPE),
                           ('other/file', tarfile.REGTYPE), ('qubit-spi-0.11.0/link', tarfile.SYMTYPE),
                           ('qubit-spi-0.11.0/hard', tarfile.LNKTYPE), ('qubit-spi-0.11.0/device', tarfile.CHRTYPE)):
            member = tarfile.TarInfo(name)
            member.type = kind
            with self.subTest(name=name), self.assertRaises(ValueError):
                check.validate_member(member, 'qubit-spi-0.11.0')

    def test_extract_regular_archive_and_reject_links(self):
        import io
        import tarfile
        import tempfile
        check = load_script('check-package')
        for unsafe in (False, True):
            data = io.BytesIO()
            with tarfile.open(fileobj=data, mode='w') as archive:
                member = tarfile.TarInfo('crate-1/src/lib.rs')
                if unsafe:
                    member.type = tarfile.SYMTYPE
                    member.linkname = '../../outside'
                    archive.addfile(member)
                else:
                    member.size = 5
                    archive.addfile(member, io.BytesIO(b'hello'))
            data.seek(0)
            with tempfile.TemporaryDirectory() as directory, tarfile.open(fileobj=data) as archive:
                destination = Path(directory)
                if unsafe:
                    with self.assertRaises(ValueError):
                        check.extract_archive(archive, destination, 'crate-1')
                    self.assertEqual([], list(destination.iterdir()))
                else:
                    check.extract_archive(archive, destination, 'crate-1')
                    self.assertEqual('hello', (destination / 'crate-1/src/lib.rs').read_text())


class CompilationTests(unittest.TestCase):
    """Exercise the actual compiler, including the previously stale API call."""

    def test_real_example_compiles_and_stale_attempt_api_fails(self):
        import tempfile
        check = load_script('check-documentation')
        root = Path(__file__).resolve().parents[2]
        scenario = {'manifests': {
            'Cargo.toml': '[workspace]\nmembers = ["app"]\nresolver = "3"\n',
            'app/Cargo.toml': '[package]\nname = "app"\nversion = "0.1.0"\nedition = "2024"\n[dependencies]\nqubit-spi = { path = "$SPI_ROOT" }\n',
        }, 'stdout': ''}
        code = '''use qubit_spi::error::ProviderAttemptFailure;
fn diagnostic(attempt: &ProviderAttemptFailure<std::io::Error>) {
    let _ = attempt.failure();
}
fn main() { let _ = diagnostic; }
'''
        with tempfile.TemporaryDirectory(prefix='spi-example-test-') as directory:
            workspace = Path(directory) / 'valid'
            block = check.ExampleBlock('probe', PurePosixPath('app/src/main.rs'), code, 'probe.md', 3)
            check.run_scenario(root, [block], scenario, workspace)
            invalid = check.ExampleBlock('probe', block.file, code.replace('attempt.failure()', 'attempt.error()'), 'probe.md', 3)
            with self.assertRaisesRegex(ValueError, 'E0599'):
                check.run_scenario(root, [invalid], scenario, Path(directory) / 'invalid')

if __name__ == '__main__':
    unittest.main()
