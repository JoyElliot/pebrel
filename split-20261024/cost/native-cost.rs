use std::{hint::black_box, mem::size_of, time::Instant};
use nebula_terminal::{Term, event::EventListener, event_loop::StreamProcessor,
    grid::Dimensions, index::{Column, Line}, inline_content::CommandView,
    render::{RenderSnapshot, SnapshotConfig}, term::{Config, cell::{Cell, CellExtra}}};

#[derive(Clone, Copy)]
pub struct Lens;
impl EventListener for Lens {
    fn command_view(&self, shell: &str, command: &str) -> Option<CommandView> {
        (shell == "pwsh:1" && command == "cat long.md").then(|| CommandView {
            mime: "text/markdown".into(), title: command.into(),
        })
    }
}
pub struct Size;
impl Dimensions for Size {
    fn total_lines(&self) -> usize { 40 }
    fn screen_lines(&self) -> usize { 40 }
    fn columns(&self) -> usize { 120 }
}
pub const START: &str = "\x1b]1337;SetUserVar=pebrel_shell=cHdzaDox\x07\x1b]1337;SetUserVar=pebrel_command=cHdzaDoxCmNhdCBsb25nLm1k\x07\x1b]133;C\x07";
pub const DONE: &str = "\x1b]133;D;0\x07";
pub fn native_payload() -> Vec<u8> {
    let mut data = Vec::new();
    for command in 0..100 {
        data.extend_from_slice(START.as_bytes());
        for row in 0..100 { data.extend_from_slice(format!("{command:03}:{row:03} {}\r\n", "x".repeat(100)).as_bytes()); }
        data.extend_from_slice(DONE.as_bytes());
    }
    data
}
fn payload(kind: &str) -> Vec<u8> {
    if kind == "native" { return native_payload(); }
    let body = match kind {
        "ascii" => "x".repeat(114),
        "cjk" => "\u{4e2d}\u{6587}\u{6d4b}".repeat(19),
        "ansi" => format!("\x1b[38;2;120;80;200m{}\x1b[0m", "x".repeat(114)),
        _ => panic!("unknown workload"),
    };
    let mut data = Vec::new();
    for row in 0..50_000 { data.extend_from_slice(format!("{row:05} {body}\r\n").as_bytes()); }
    data
}
fn main() {
    let kind = std::env::args().nth(1).unwrap();
    let post_native = kind.starts_with("post-");
    let data = payload(kind.strip_prefix("post-").unwrap_or(&kind));
    let mut term = Term::new(Config { scrolling_history: 10_000, ..Config::default() }, &Size, Lens);
    let mut stream = StreamProcessor::default();
    if post_native {
        stream.feed(&mut term, &Lens, format!("{START}# Seed result\r\n{DONE}").as_bytes());
        let marker = term.grid()[Line(0)][Column(0)].inline_marker().unwrap();
        assert_eq!(marker.surface.update_since(0).chunks[0].text, "# Seed result\n");
    }
    let start = Instant::now();
    for chunk in data.chunks(64 * 1024) { stream.feed(&mut term, &Lens, chunk); }
    let feed_us = start.elapsed().as_secs_f64() * 1e6;
    let cfg = SnapshotConfig { rows: 40, cols: 120 };
    let start = Instant::now();
    let mut text_cells = 0;
    for _ in 0..100 {
        let snapshot = RenderSnapshot::capture(&term, &cfg);
        text_cells = snapshot.segments.iter().map(|segment| segment.cells.len()).sum::<usize>();
        black_box(&snapshot);
    }
    let snapshot_us = start.elapsed().as_secs_f64() * 1e6 / 100.0;
    let markers = (0..40).filter(|&row| term.grid()[Line(row)][Column(0)].inline_marker().is_some()).count();
    if kind == "native" { assert!(markers > 0); }
    let mut checksum = 0u64;
    for row in 0..40 { for column in 0..120 {
        checksum = checksum.wrapping_add(term.grid()[Line(row)][Column(column)].c as u64);
    }}
    println!("{}", serde_json::json!({"workload": kind, "feed_us": feed_us,
        "snapshot_us": snapshot_us, "bytes": data.len(), "checksum": checksum,
        "cells": text_cells, "visible_markers": markers, "real_native_capture_seeded": post_native,
        "cell_size": size_of::<Cell>(), "extra_size": size_of::<CellExtra>(),
        "row_size": size_of::<nebula_terminal::grid::Row<Cell>>(),
        "grid_size": size_of::<nebula_terminal::grid::Grid<Cell>>(),
        "cursor": [term.grid().cursor.point.line.0, term.grid().cursor.point.column.0 as i32]}));
}
