use std::sync::Arc;
use nebula_terminal::{Term, event::EventListener, event_loop::StreamProcessor, grid::Dimensions, index::{Column, Line}, inline_content::CommandView, term::Config};
#[derive(Clone, Copy)] struct Lens;
impl EventListener for Lens {
    fn command_view(&self, _: &str, _: &str) -> Option<CommandView> {
        Some(CommandView { mime: "text/markdown".into(), title: "test".into() })
    }
}
struct Size;
impl Dimensions for Size {
    fn total_lines(&self) -> usize { 3 }
    fn screen_lines(&self) -> usize { 3 }
    fn columns(&self) -> usize { 20 }
}
#[test]
fn cached_native_owner_dies_when_its_last_logical_row_leaves() {
    let mut term = Term::new(Config { scrolling_history: 1, ..Config::default() }, &Size, Lens);
    let mut stream = StreamProcessor::default();
    let input = b"\x1b]1337;SetUserVar=pebrel_shell=cHdzaDox\x07\x1b]1337;SetUserVar=pebrel_command=cHdzaDoxCmNhdCBsb25nLm1k\x07\x1b]133;C\x07# marker\r\n\x1b]133;D;0\x07";
    stream.feed(&mut term, &Lens, input);
    let owner = Arc::downgrade(&term.grid()[Line(0)][Column(0)].inline_marker().expect("actual command capture").surface);
    let mut found_eviction = false;
    for _ in 0..10 {
        stream.feed(&mut term, &Lens, b"plain\r\n");
        let logical_markers = (-((term.grid().total_lines() - term.grid().screen_lines()) as i32)..term.grid().screen_lines() as i32)
            .filter(|row| term.grid()[Line(*row)][Column(0)].inline_marker().is_some()).count();
        if logical_markers == 0 {
            found_eviction = true;
            assert!(owner.upgrade().is_none(), "no logical markers remain, but cached row still owns the native surface");
            break;
        }
    }
    assert!(found_eviction, "fixture must drive last marker outside logical grid");
}
