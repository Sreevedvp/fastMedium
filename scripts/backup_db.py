#!/usr/bin/env python3
"""Create a consistent SQLite backup, including committed WAL data, without overwriting files."""
import argparse
from pathlib import Path
import sqlite3

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('source', type=Path)
parser.add_argument('destination', type=Path)
args = parser.parse_args()
source, destination = args.source.resolve(), args.destination.resolve()
if not source.is_file():
    parser.error('Source database does not exist.')
try:
    # Exclusive creation prevents accidental replacement of a previous backup.
    with destination.open('xb'):
        pass
except FileExistsError:
    parser.error('Destination already exists; choose a new backup filename.')
try:
    with sqlite3.connect(source.as_uri() + '?mode=ro', uri=True, timeout=10) as src, sqlite3.connect(destination) as dst:
        src.backup(dst)
        if dst.execute('PRAGMA integrity_check').fetchall() != [('ok',)]:
            raise RuntimeError('Backup integrity check failed.')
        if dst.execute('PRAGMA foreign_key_check').fetchall():
            raise RuntimeError('Backup contains foreign key violations.')
    destination.chmod(0o600)
except BaseException:
    destination.unlink(missing_ok=True)
    raise
print(f'Verified SQLite backup: {destination}')
