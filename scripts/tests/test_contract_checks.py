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

    def test_example_source_denies_unused_must_use_only_at_crate_roots(self):
        source = self.check.example_source
        attribute = '#![deny(unused_must_use)]\n'
        self.assertEqual(attribute + 'fn main() {}\n', source(PurePosixPath('app/src/main.rs'), 'fn main() {}\n'))
        self.assertEqual(attribute + 'pub fn greet() {}\n', source(PurePosixPath('lib/src/lib.rs'), 'pub fn greet() {}\n'))
        self.assertEqual('pub fn greet() {}\n', source(PurePosixPath('app/src/providers.rs'), 'pub fn greet() {}\n'))
        self.assertEqual(attribute + '#![allow(dead_code)]\nfn main() {}\n',
                         source(PurePosixPath('app/src/main.rs'), '#![allow(dead_code)]\nfn main() {}\n'))
        self.assertEqual('#!/usr/bin/env rustx\n' + attribute + 'fn main() {}\n',
                         source(PurePosixPath('app/src/main.rs'), '#!/usr/bin/env rustx\nfn main() {}\n'))

    def test_unmarked_rust_fences_are_not_executable_examples(self):
        blocks = self.check.extract_blocks('```rust\nfn main() {}\n```\n', 'README.md')
        self.assertEqual([], blocks)

    def test_reject_unclosed_fence_and_unused_marker(self):
        for source in ('```text\nunclosed\n', '<!-- spi-example: a; file: app/src/main.rs -->\n'):
            with self.subTest(source=source), self.assertRaises(ValueError):
                self.check.extract_blocks(source, 'README.md')

    def test_reject_non_plain_rust_fence_for_marked_examples(self):
        for language in ('rust no_run', 'rust ignore', 'rust,no_run'):
            source = f'<!-- spi-example: quick-start; file: app/src/main.rs -->\n```{language}\nfn main() {{}}\n```\n'
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

    def test_compose_base_and_inventory_sources(self):
        base = [
            self.check.ExampleBlock('three-crates', PurePosixPath('lib/src/lib.rs'),
                                    'pub struct Spec;\n', 'README.md', 10),
            self.check.ExampleBlock('three-crates', PurePosixPath('app/src/main.rs'),
                                    'fn main() { register(); }\n', 'README.md', 20),
        ]
        delta = [
            self.check.ExampleBlock('readme-inventory', PurePosixPath('lib/src/lib.rs'),
                                    'pub mod providers {}\n', 'README.md', 30),
            self.check.ExampleBlock('readme-inventory', PurePosixPath('app/src/main.rs'),
                                    'fn main() { build_registry(); }\n', 'README.md', 40),
            self.check.ExampleBlock('readme-inventory', PurePosixPath('app/src/providers.rs'),
                                    'use provider as _;\n', 'README.md', 50),
        ]
        sources = self.check.compose_sources(delta, base, ['lib/src/lib.rs'])
        self.assertEqual('pub struct Spec;\n\npub mod providers {}\n',
                         sources[PurePosixPath('lib/src/lib.rs')])
        self.assertEqual(delta[1].code, sources[PurePosixPath('app/src/main.rs')])
        self.assertEqual(delta[2].code, sources[PurePosixPath('app/src/providers.rs')])
        self.assertEqual('pub struct Spec;\n', base[0].code)

    def test_compose_rejects_invalid_append_configuration(self):
        base = [self.check.ExampleBlock('base', PurePosixPath('app/src/main.rs'), 'base', 'README.md', 1)]
        delta = [self.check.ExampleBlock('delta', PurePosixPath('app/src/main.rs'), 'delta', 'README.md', 2)]
        only_base = [self.check.ExampleBlock('base', PurePosixPath('base.rs'), 'base', 'README.md', 1)]
        only_delta = [self.check.ExampleBlock('delta', PurePosixPath('delta.rs'), 'delta', 'README.md', 2)]
        invalid = (['app/src/main.rs', 'app/src/main.rs'], ['../escape.rs'], ['missing.rs'], [42])
        for append_files in invalid:
            with self.subTest(append_files=append_files), self.assertRaises(ValueError):
                self.check.compose_sources(delta, base, append_files)
        for delta_blocks, base_blocks in ((delta, ()), ((), base)):
            with self.subTest(delta=delta_blocks, base=base_blocks), self.assertRaises(ValueError):
                self.check.compose_sources(delta_blocks, base_blocks, ['app/src/main.rs'])
        with self.assertRaises(ValueError):
            self.check.compose_sources(only_delta, only_base, ['base.rs'])

    def test_check_documentation_rejects_invalid_base_scenarios_before_cargo(self):
        import json
        import tempfile

        cases = (
            ({'readme-inventory': []}, {'readme-inventory': {'base_scenario': 'missing'}},
             'invalid base scenario'),
            ({'readme-inventory': []}, {'readme-inventory': {'base_scenario': 'readme-inventory'}},
             'invalid base scenario'),
            ({'readme-inventory': [], 'base-one': [], 'base-two': []}, {
                'readme-inventory': {'base_scenario': 'base-one'},
                'base-one': {'base_scenario': 'base-two'},
                'base-two': {},
            }, 'invalid base scenario'),
        )
        for documents, scenarios, error in cases:
            with self.subTest(scenarios=scenarios), tempfile.TemporaryDirectory(prefix='spi-base-scenario-') as directory:
                root = Path(directory)
                (root / 'README.md').write_text('', encoding='utf-8')
                fixture = root / 'tests/fixtures/documentation_examples/scenarios.json'
                fixture.parent.mkdir(parents=True)
                fixture.write_text(json.dumps({
                    'documents': {'README.md': documents},
                    'scenarios': scenarios,
                }), encoding='utf-8')
                with self.assertRaisesRegex(ValueError, error):
                    self.check.check_documentation(root)

    def test_each_readme_requires_every_inventory_block(self):
        root = Path(__file__).resolve().parents[2]
        expected_files = ('lib-greeter/src/lib.rs', 'lib-friendly-greeter/src/lib.rs',
                          'app/src/greeter_providers.rs', 'app/src/main.rs')
        for document in ('README.md', 'README.zh_CN.md'):
            source = (root / document).read_text(encoding='utf-8')
            for relative in expected_files:
                with self.subTest(document=document, file=relative):
                    marker = f'<!-- spi-example: readme-inventory; file: {relative} -->\n'
                    self.assertIn(marker, source)
                    mutated = source.replace(marker, '', 1)
                    blocks = self.check.extract_blocks(mutated, document)
                    with self.assertRaisesRegex(ValueError, 'example inventory mismatch'):
                        self.check.validate_blocks(blocks, {'three-crates': [
                            'lib-greeter/src/lib.rs', 'lib-foo/src/lib.rs',
                            'lib-friendly-greeter/src/lib.rs', 'app/src/main.rs'],
                            'readme-inventory': list(expected_files)}, document)

    def test_actual_readmes_declare_inventory_and_base_sources(self):
        import json
        root = Path(__file__).resolve().parents[2]
        manifest = json.loads((root / 'tests/fixtures/documentation_examples/scenarios.json').read_text())
        expected = {
            'lib-greeter/src/lib.rs', 'lib-friendly-greeter/src/lib.rs',
            'app/src/greeter_providers.rs', 'app/src/main.rs',
        }
        for document in ('README.md', 'README.zh_CN.md'):
            blocks = self.check.extract_blocks((root / document).read_text(encoding='utf-8'), document)
            self.assertEqual(expected, {str(block.file) for block in blocks if block.scenario == 'readme-inventory'})
            self.assertEqual({'lib-greeter/src/lib.rs', 'lib-foo/src/lib.rs',
                              'lib-friendly-greeter/src/lib.rs', 'app/src/main.rs'},
                             {str(block.file) for block in blocks if block.scenario == 'three-crates'})
            self.assertIn('readme-inventory', manifest['documents'][document])



class ArchiveTests(unittest.TestCase):
    """Archives must be self-contained regular files below the crate root."""

    def test_archive_test_command_excludes_checkout_only_fixtures(self):
        check = load_script('check-package')
        command = check.package_test_command()
        self.assertEqual(command[:4], ['cargo', 'test', '--locked', '--all-features'])
        self.assertEqual(command[4:], [
            '--', '--skip', 'inventory::cross_crate_inventory_tests',
            '--skip', 'inventory::fixture_lock_tests',
        ])

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

    def test_unused_result_in_example_is_a_compile_error(self):
        import tempfile
        check = load_script('check-documentation')
        root = Path(__file__).resolve().parents[2]
        scenario = {'manifests': {
            'Cargo.toml': '[workspace]\nmembers = ["app"]\nresolver = "3"\n',
            'app/Cargo.toml': '[package]\nname = "app"\nversion = "0.1.0"\nedition = "2024"\n[dependencies]\nqubit-spi = { path = "$SPI_ROOT" }\n',
        }, 'stdout': ''}
        code = 'fn main() { std::fs::read("missing"); }\n'
        with tempfile.TemporaryDirectory(prefix='spi-unused-result-') as directory:
            block = check.ExampleBlock('probe', PurePosixPath('app/src/main.rs'), code, 'probe.md', 3)
            with self.assertRaisesRegex(ValueError, 'unused') as error:
                check.run_scenario(root, [block], scenario, Path(directory) / 'invalid')
            self.assertIn('Result', str(error.exception))

    def test_readme_inventory_stale_method_fails_for_each_language(self):
        import json
        import tempfile
        check = load_script('check-documentation')
        root = Path(__file__).resolve().parents[2]
        manifest = json.loads((root / 'tests/fixtures/documentation_examples/scenarios.json').read_text())
        with tempfile.TemporaryDirectory(prefix='spi-readme-inventory-') as directory:
            for document in ('README.md', 'README.zh_CN.md'):
                with self.subTest(document=document):
                    source = (root / document).read_text(encoding='utf-8')
                    mutated = source.replace('providers::build_registry()?',
                                             'providers::build_registry()?.nonexistent_method()?')
                    self.assertNotEqual(source, mutated)
                    blocks = check.extract_blocks(mutated, document)
                    check.validate_blocks(blocks, manifest['documents'][document], document)
                    selected = [block for block in blocks if block.scenario == 'readme-inventory']
                    base = [block for block in blocks if block.scenario == 'three-crates']
                    with self.assertRaisesRegex(ValueError, 'E0599'):
                        check.run_scenario(root, selected, manifest['scenarios']['readme-inventory'],
                                           Path(directory) / document.replace('/', '_'), base_blocks=base)

if __name__ == '__main__':
    unittest.main()
