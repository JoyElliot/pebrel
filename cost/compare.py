from pathlib import Path
import subprocess, json, statistics, hashlib
ROOT = Path(__file__).resolve().parent
samples = []

def run(variant, workload):
    binary = 'text-cost' if workload in ('ascii', 'cjk', 'ansi') else 'image-cost'
    result = json.loads(subprocess.check_output([str(ROOT / variant / 'target/release' / (binary+'.exe')), workload], text=True))
    result['variant'] = variant
    return result

for workload in ['ascii', 'cjk', 'ansi', 'image', 'mixed']:
    variants = ['base', 'review', 'fixed'] if workload in ('ascii', 'cjk', 'ansi') else ['review', 'fixed']
    for variant in variants: run(variant, workload)
    for i in range(9):
        order = variants[i % len(variants):] + variants[:i % len(variants)]
        if (i // len(variants)) % 2: order = order[::-1]
        for variant in order:
            result = run(variant, workload)
            result['iteration'] = i
            samples.append(result)
    observed = [s for s in samples if s['workload'] == workload]
    for field in (['checksum','cells','bytes','cursor'] if len(variants) == 3 else ['tiles']):
        assert all(s[field] == observed[0][field] for s in observed), (workload, field)

(ROOT / 'samples.json').write_text(json.dumps(samples, indent=2), encoding='utf-8')
summary = []
for workload in ['ascii','cjk','ansi','image','mixed']:
    for variant in ['base','review','fixed']:
        group = [s for s in samples if s['workload']==workload and s['variant']==variant]
        if not group: continue
        row = {'workload':workload,'variant':variant,'n':len(group)}
        for field in ['feed_us','snapshot_us']:
            values = [s[field] for s in group]
            row[field] = {'median':statistics.median(values),'min':min(values),'max':max(values)}
        for field in ['checksum','cells','bytes','tiles','cell_size','extra_size','row_size','grid_size','term_size']:
            if field in group[0]: row[field] = group[0][field]
        summary.append(row)
(ROOT / 'summary.json').write_text(json.dumps(summary,indent=2),encoding='utf-8')
for row in summary:
    f,s = row['feed_us'],row['snapshot_us']
    print(f"{row['workload']:6} {row['variant']:10} feed={f['median']/1000:.3f}ms [{f['min']/1000:.3f},{f['max']/1000:.3f}] snapshot={s['median']:.2f}us [{s['min']:.2f},{s['max']:.2f}]")
for label in ['base','review','fixed']:
    print(label, 'lock', hashlib.sha256((ROOT/label/'Cargo.lock').read_bytes()).hexdigest())
