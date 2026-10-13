use std::{hint::black_box, mem::size_of, time::Instant};
use nebula_terminal::{Term, event::VoidListener, event_loop::StreamProcessor,
    grid::Dimensions, index::{Column, Line}, render::{RenderSnapshot, SnapshotConfig},
    term::{Config, cell::{Cell, CellExtra}}};

struct Size;
impl Dimensions for Size {
    fn total_lines(&self) -> usize { 40 }
    fn screen_lines(&self) -> usize { 40 }
    fn columns(&self) -> usize { 120 }
}

fn payload(kind: &str) -> Vec<u8> {
    // Escaped scalars make the CJK fixture independent of shell source encoding.
    let body = match kind {
        "ascii" => "x".repeat(114),
        "cjk" => "\u{4e2d}\u{6587}\u{6d4b}".repeat(19),
        "ansi" => format!("\x1b[38;2;120;80;200m{}\x1b[0m", "x".repeat(114)),
        _ => panic!("unknown workload"),
    };
    if kind == "cjk" { assert_eq!(&body.as_bytes()[..3], &[0xe4, 0xb8, 0xad]); }
    let mut data = Vec::new();
    for n in 0..50_000 { data.extend_from_slice(format!("{n:05} {body}\r\n").as_bytes()); }
    data
}

fn main() {
    let kind = std::env::args().nth(1).unwrap();
    let data = payload(&kind);
    let mut term = Term::new(Config { scrolling_history: 10_000, ..Config::default() }, &Size, VoidListener);
    let mut stream = StreamProcessor::default();
    let start = Instant::now();
    for chunk in data.chunks(64 * 1024) { stream.feed(&mut term, &VoidListener, chunk); }
    let feed_us = start.elapsed().as_secs_f64() * 1e6;
    let cfg = SnapshotConfig { rows: 40, cols: 120 };
    let start = Instant::now();
    let mut text_cells = 0;
    for _ in 0..50 {
        let snap = RenderSnapshot::capture(&term, &cfg);
        text_cells = snap.segments.iter().map(|s| s.cells.len()).sum::<usize>();
        black_box(&snap);
    }
    let snapshot_us = start.elapsed().as_secs_f64() * 1e6 / 50.0;
    let mut checksum = 0u64;
    for row in 0..40 { for col in 0..120 {
        checksum = checksum.wrapping_add(term.grid()[Line(row)][Column(col)].c as u64);
    }}
    println!("{}", serde_json::json!({"workload": kind, "feed_us":feed_us,
        "snapshot_us":snapshot_us,"bytes":data.len(),"checksum":checksum,"cells":text_cells,
        "cell_size":size_of::<Cell>(),"extra_size":size_of::<CellExtra>(),
        "row_size":size_of::<nebula_terminal::grid::Row<Cell>>(),
        "grid_size":size_of::<nebula_terminal::grid::Grid<Cell>>(),"term_size":size_of::<Term<VoidListener>>(),
        "cursor":[term.grid().cursor.point.line.0,term.grid().cursor.point.column.0 as i32]}));
}
