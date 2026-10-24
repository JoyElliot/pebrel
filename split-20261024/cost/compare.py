from pathlib import Path
import hashlib
import json
import statistics
import subprocess

root = Path(__file__).resolve().parent
variants = ["main", "a", "b"]
kinds = ["ascii", "cjk", "ansi", "native", "post-ascii", "post-cjk", "post-ansi"]
fields = ["bytes", "checksum", "cursor", "cells", "visible_markers", "real_native_capture_seeded", "cell_size", "extra_size", "row_size"]


def run(variant, executable, *arguments):
    path = root / variant / "bin" / (executable + ".exe")
    result = subprocess.run([str(path), *arguments], capture_output=True, text=True, check=True)
    return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]


def summarize(samples, workload_key, metrics):
    summary = {}
    for workload in dict.fromkeys(sample[workload_key] for sample in samples):
        entry = {}
        for variant in variants:
            selected = [sample for sample in samples if sample["variant"] == variant and sample[workload_key] == workload]
            entry[variant] = {
                metric: {
                    "median": statistics.median(sample[metric] for sample in selected),
                    "min": min(sample[metric] for sample in selected),
                    "max": max(sample[metric] for sample in selected),
                }
                for metric in metrics
            }
        entry["delta_percent"] = {
            variant: {metric: (entry[variant][metric]["median"] / entry["main"][metric]["median"] - 1) * 100 for metric in metrics}
            for variant in ["a", "b"]
        }
        summary[workload] = entry
    return summary


warmup = []
for kind in kinds:
    for variant in variants:
        warmup.append({**run(variant, "native-cost", kind)[0], "variant": variant})
samples = []
for round_index in range(9):
    offset = round_index % len(variants)
    order = variants[offset:] + variants[:offset]
    if round_index % 2:
        order = order[::-1]
    workload_offset = round_index % len(kinds)
    for kind in kinds[workload_offset:] + kinds[:workload_offset]:
        group = [{**run(variant, "native-cost", kind)[0], "variant": variant, "round": round_index} for variant in order]
        for field in fields:
            assert all(sample[field] == group[0][field] for sample in group), (kind, field)
        samples.extend(group)
        (root / "samples.json").write_text(json.dumps(samples, indent=2) + "\n")
(root / "warmup.json").write_text(json.dumps(warmup, indent=2) + "\n")
summary = summarize(samples, "workload", ["feed_us", "snapshot_us"])
(root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")

validation_warmup = [{**run(variant, "native-validate")[0], "variant": variant} for variant in variants]
validation = []
for round_index in range(9):
    offset = round_index % len(variants)
    order = variants[offset:] + variants[:offset]
    if round_index % 2:
        order = order[::-1]
    for variant in order:
        sample = run(variant, "native-validate")[0]
        assert sample["rows"] == 39 and sample["matched"] == 39000
        validation.append({**sample, "variant": variant, "round": round_index, "case": "source-validation"})
(root / "validation-warmup.json").write_text(json.dumps(validation_warmup, indent=2) + "\n")
(root / "validation-samples.json").write_text(json.dumps(validation, indent=2) + "\n")
(root / "validation-summary.json").write_text(json.dumps(summarize(validation, "case", ["validation_us"]), indent=2) + "\n")

memory = []
for variant in variants:
    memory.extend({**sample, "variant": variant} for sample in run(variant, "native-memory"))
(root / "memory-results.json").write_text(json.dumps(memory, indent=2) + "\n")
binary_hashes = {variant: {name: hashlib.sha256((root / variant / "bin" / name).read_bytes()).hexdigest() for name in ["native-cost.exe", "native-memory.exe", "native-validate.exe"]} for variant in variants}
(root / "binary-hashes.json").write_text(json.dumps(binary_hashes, indent=2) + "\n")
print(json.dumps({"samples": len(samples), "warmup": len(warmup), "validation": len(validation), "memory_cases": len(memory)}))
