use std::{hint::black_box, time::Instant};
use nebula_terminal::{Term, event_loop::StreamProcessor, index::{Column, Line}, term::Config};
#[path="native-cost.rs"] mod fixture;
use fixture::{Lens, Size};
fn main() {
    let data = fixture::native_payload();
    let mut term = Term::new(Config { scrolling_history: 10_000, ..Config::default() }, &Size, Lens);
    let mut stream = StreamProcessor::default();
    for chunk in data.chunks(64 * 1024) { stream.feed(&mut term, &Lens, chunk); }
    let rows = (0..40).filter(|&row| term.grid()[Line(row)][Column(0)].inline_marker().is_some()).count();
    assert_eq!(rows, 39);
    let start = Instant::now();
    let mut matched = 0usize;
    for _ in 0..1000 { for row in 0..40 {
        let cells = &term.grid()[Line(row)][..];
        if let Some(marker) = cells[0].inline_marker() {
            assert!(marker.surface.source_row_matches(marker.row, black_box(cells)));
            matched += 1;
        }
    }}
    let validation_us = start.elapsed().as_secs_f64() * 1e6 / 1000.0;
    assert_eq!(matched, rows * 1000);
    println!("{}", serde_json::json!({"validation_us":validation_us,
        "rows":rows,"matched":matched,"scope":"core source_row_matches for 39 visible native result rows; not GUI frame time"}));
}
