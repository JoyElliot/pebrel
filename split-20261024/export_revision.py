import hashlib
import io
import json
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

repo, reference, destination, base, runtime = sys.argv[1:]
repo, destination, runtime = Path(repo), Path(destination), Path(runtime)
commit = subprocess.check_output(["git", "rev-parse", reference], cwd=repo, text=True).strip()
tree = subprocess.check_output(["git", "rev-parse", commit + "^{tree}"], cwd=repo, text=True).strip()
source = destination / "source"
source.mkdir(parents=True, exist_ok=False)
archive = subprocess.check_output(["git", "archive", commit], cwd=repo)
with tarfile.open(fileobj=io.BytesIO(archive)) as bundle:
    bundle.extractall(source, filter="data")
files = {}
for line in subprocess.check_output(["git", "ls-tree", "-r", "--full-tree", commit], cwd=repo, text=True).splitlines():
    metadata, name = line.split("\t", 1)
    content = (source / name).read_bytes()
    blob = hashlib.sha1(b"blob " + str(len(content)).encode() + b"\0" + content).hexdigest()
    assert blob == metadata.split()[2], name
    files[name] = hashlib.sha256(content).hexdigest()
for name in ["conpty.dll", "OpenConsole.exe"]:
    target = source / "assets/windows/conhost" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(runtime / name, target)
manifest = {"commit": commit, "tree": tree, "base": base, "files": files}
(destination / "source-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(json.dumps({"commit": commit, "tree": tree, "verified_files": len(files)}), flush=True)
