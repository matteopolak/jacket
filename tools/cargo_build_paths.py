"""Use Cargo's real configuration without redirecting builds to another volume."""
from dataclasses import dataclass
import json
from pathlib import Path
import shutil
import subprocess
import tomllib

PROTECTED = ('reference', 'corpus', 'vendor', '.git')


def checked_directory(path, root):
    directory = Path(path)
    if not directory.is_absolute():
        directory = root / directory
    directory = directory.resolve()
    if any(directory.is_relative_to((root / name).resolve()) for name in PROTECTED):
        raise ValueError('build directory is inside protected original inputs')
    return directory


def pinned_cargo_command(root, environment):
    channel = tomllib.loads((root / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    if not isinstance(channel, str) or not channel.strip():
        raise ValueError('rust-toolchain.toml must select a pinned toolchain channel')
    rustup = shutil.which('rustup', path=environment.get('PATH'))
    if rustup is None:
        raise ValueError('the independently installed rustup is required')
    executable = Path(rustup).resolve(strict=True)
    if any(executable.is_relative_to((root / name).resolve()) for name in PROTECTED):
        raise ValueError('rustup resolves into protected original inputs')
    return [str(executable), 'run', channel, 'cargo']


@dataclass(frozen=True)
class TargetDirectory:
    path: Path
    source: str
    metadata_command: tuple[str, ...] = ()

    def receipt(self):
        return {'path': str(self.path), 'source': self.source,
                'metadata_command': list(self.metadata_command)}


def configured_target_directory(argument, environment, cargo, root):
    """Resolve CLI, environment, then locked/offline Cargo configuration."""
    if argument is not None:
        if not str(argument).strip():
            raise ValueError('--target-dir must not be empty')
        return TargetDirectory(checked_directory(argument, root), 'command-line')
    if 'CARGO_TARGET_DIR' in environment:
        value = environment['CARGO_TARGET_DIR']
        if not value.strip():
            raise ValueError('CARGO_TARGET_DIR must not be empty')
        return TargetDirectory(checked_directory(value, root), 'environment')
    command = [*cargo, 'metadata', '--format-version', '1', '--no-deps', '--locked', '--offline']
    metadata = json.loads(subprocess.check_output(command, cwd=root, env=environment, text=True))
    value = metadata.get('target_directory')
    if not isinstance(value, str) or not value.strip() or not Path(value).is_absolute():
        raise ValueError('Cargo metadata must report an absolute target_directory')
    return TargetDirectory(checked_directory(value, root), 'cargo-metadata', tuple(command))
