from pathlib import Path
import io, json, subprocess, sys, tarfile
root = Path(__file__).resolve().parent
checkout = Path(sys.argv[1]).resolve()
for variant, revision in json.loads((root / "revisions.json").read_text()).items():
    destination = root / "sources" / variant
    destination.mkdir(parents=True, exist_ok=False)
    archive = subprocess.check_output(["git", "-C", str(checkout), "archive", revision, "nebula_terminal"])
    with tarfile.open(fileobj=io.BytesIO(archive)) as contents:
        contents.extractall(destination, filter="data")
    (destination / "Cargo.toml").write_text((root / "source-workspace.toml").read_text())
    print(variant, revision, destination)
