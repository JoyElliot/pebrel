from pathlib import Path
import json, statistics, subprocess
root = Path(__file__).resolve().parent

def run(variant):
    binary = root / variant / "target/release/native-validate.exe"
    result = subprocess.run([str(binary)], capture_output=True, text=True, check=True)
    return {"variant": variant, **json.loads(result.stdout)}

for variant in ["main", "merged"]:
    run(variant)
samples = []
for round_index in range(9):
    order = ["main", "merged"] if round_index % 2 == 0 else ["merged", "main"]
    pair = [run(variant) for variant in order]
    assert pair[0]["matched"] == pair[1]["matched"] == 39000
    samples.extend({**sample, "round": round_index} for sample in pair)
summary = {}
for variant in ["main", "merged"]:
    values = [sample["validation_us"] for sample in samples if sample["variant"] == variant]
    summary[variant] = {"median": statistics.median(values), "min": min(values), "max": max(values)}
summary["delta_percent"] = (summary["merged"]["median"] / summary["main"]["median"] - 1) * 100
(root / "validation-samples.json").write_text(json.dumps(samples, indent=2) + "\n")
(root / "validation-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
