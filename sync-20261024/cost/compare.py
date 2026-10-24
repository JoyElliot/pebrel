from pathlib import Path
import subprocess, json, statistics

root = Path(__file__).resolve().parent
kinds = ["ascii", "cjk", "ansi", "native", "post-ascii", "post-cjk", "post-ansi"]
binaries = {variant: root / variant / "target/release/native-cost.exe" for variant in ["main", "merged"]}

def run(variant, kind):
    result = subprocess.run([str(binaries[variant]), kind], capture_output=True, text=True, check=True)
    return {**json.loads(result.stdout.strip()), "variant": variant}

warmup = [run(variant, kind) for kind in kinds for variant in binaries]
(root / "warmup.json").write_text(json.dumps(warmup, indent=2) + "\n")
samples = []
for round_index in range(9):
    order = ["main", "merged"] if round_index % 2 == 0 else ["merged", "main"]
    offset = round_index % len(kinds)
    for kind in kinds[offset:] + kinds[:offset]:
        pair = [run(variant, kind) for variant in order]
        for field in ["bytes", "checksum", "cursor", "cells", "visible_markers", "real_native_capture_seeded", "cell_size", "extra_size", "row_size"]:
            assert pair[0][field] == pair[1][field], (kind, field, pair)
        samples.extend({**sample, "round": round_index} for sample in pair)
(root / "samples.json").write_text(json.dumps(samples, indent=2) + "\n")
summary = {}
for kind in kinds:
    entry = {}
    for variant in binaries:
        selected = [sample for sample in samples if sample["variant"] == variant and sample["workload"] == kind]
        entry[variant] = {metric: {"median": statistics.median(sample[metric] for sample in selected), "min": min(sample[metric] for sample in selected), "max": max(sample[metric] for sample in selected)} for metric in ["feed_us", "snapshot_us"]}
    for metric in ["feed_us", "snapshot_us"]:
        entry[metric + "_delta_percent"] = (entry["merged"][metric]["median"] / entry["main"][metric]["median"] - 1) * 100
    summary[kind] = entry
(root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
for kind, entry in summary.items():
    print(kind, "feed delta percent", round(entry["feed_us_delta_percent"], 2))
print("Measured samples", len(samples), "plus warmups", len(warmup))
