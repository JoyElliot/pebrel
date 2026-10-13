use std::{hint::black_box, time::Instant};
use nebula_terminal::{Term, event::{VoidListener, WindowSize}, event_loop::StreamProcessor,
    grid::Dimensions, render::{RenderSnapshot, SnapshotConfig}, term::Config};
struct Size;
impl Dimensions for Size {
    fn total_lines(&self) -> usize { 40 }
    fn screen_lines(&self) -> usize { 40 }
    fn columns(&self) -> usize { 120 }
}
fn main() {
    let kind = std::env::args().nth(1).unwrap();
    assert!(kind == "image" || kind == "mixed");
    let image = "\x1b]1337;File=inline=1;width=40;height=4;preserveAspectRatio=0:iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLttAAAAABJRU5ErkJggg==\x07";
    let mut data = Vec::new();
    for n in 0..1000 {
        data.extend_from_slice(image.as_bytes());
        if kind == "mixed" { data.extend_from_slice(format!("\x1b[31m{n:04} \u{4e2d}\u{6587} e\u{301}\x1b[0m").as_bytes()); }
        data.extend_from_slice(b"\r\n");
    }
    let mut term = Term::new(Config { scrolling_history: 100, ..Config::default() }, &Size, VoidListener);
    let mut stream = StreamProcessor::default();
    stream.resize(WindowSize { num_cols:120, num_lines:40, cell_width:10, cell_height:20 });
    let start = Instant::now();
    for bytes in data.chunks(64*1024) { stream.feed(&mut term, &VoidListener, bytes); }
    let feed_us = start.elapsed().as_secs_f64()*1e6;
    let cfg = SnapshotConfig { rows:40, cols:120 };
    let mut tiles = 0;
    let start = Instant::now();
    for _ in 0..50 {
        let snapshot = RenderSnapshot::capture(&term, &cfg);
        tiles = snapshot.image_runs.iter().map(|r| r.columns as usize).sum::<usize>();
        black_box(&snapshot);
    }
    let snapshot_us = start.elapsed().as_secs_f64()*1e6/50.0;
    assert_eq!(tiles, 39*40);
    stream.feed(&mut term, &VoidListener, b"\x1b[2J\x1b[3J");
    assert!(RenderSnapshot::capture(&term, &cfg).image_runs.is_empty());
    println!("{}", serde_json::json!({"workload":kind,"feed_us":feed_us,"snapshot_us":snapshot_us,"tiles":tiles}));
}
