"""Export the measured terminal revisions from a local Pebrel Git checkout.

Usage: python prepare.py /path/to/pebrel
Requires Python 3.12+ and Git. Run in a fresh evidence checkout.
Build/run commands are in ../commands.txt.
"""

import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile

root = Path(__file__).resolve().parent
checkout = Path(sys.argv[1]).resolve()
revisions = json.loads((root / "revisions.json").read_text())
for variant, revision in revisions.items():
    destination = root / "sources" / variant
    destination.mkdir(parents=True, exist_ok=False)
    archive = subprocess.check_output(
        ["git", "-C", str(checkout), "archive", revision, "nebula_terminal"]
    )
    with tarfile.open(fileobj=io.BytesIO(archive)) as contents:
        contents.extractall(destination, filter="data")
    shutil.copyfile(root / "source-workspace.toml", destination / "Cargo.toml")
    print(variant, revision, destination)
