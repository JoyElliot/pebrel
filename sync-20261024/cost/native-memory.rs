use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};
use nebula_terminal::{Term, event_loop::StreamProcessor, index::{Line, Column}, term::Config};
#[path="native-cost.rs"] mod fixture;
use fixture::{Lens, Size};
static LIVE: AtomicIsize = AtomicIsize::new(0);
struct Counted;
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() { LIVE.fetch_add(layout.size() as isize, Ordering::Relaxed); }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(ptr, layout, size) };
        if !next.is_null() { LIVE.fetch_add(size as isize - layout.size() as isize, Ordering::Relaxed); }
        next
    }
}
#[global_allocator] static ALLOC: Counted = Counted;
fn live() -> isize { LIVE.load(Ordering::Relaxed) }
fn main() {
    let data = fixture::native_payload();
    let before = live();
    let mut term = Term::new(Config { scrolling_history: 10_000, ..Config::default() }, &Size, Lens);
    let mut stream = StreamProcessor::default();
    for chunk in data.chunks(64 * 1024) { stream.feed(&mut term, &Lens, chunk); }
    assert!(term.grid()[Line(0)][Column(0)].inline_marker().is_some());
    let fill = live() - before;
    stream.feed(&mut term, &Lens, b"\x1b[3J");
    let clear_history = live() - before;
    stream.feed(&mut term, &Lens, b"\x1bc");
    let reset = live() - before;
    drop(stream); drop(term);
    let after_drop = live() - before;
    assert_eq!(after_drop, 0);
    println!("{}", serde_json::json!({"case":"real_native_capture", "fill":fill,
        "clear_history":clear_history, "reset":reset, "after_drop":after_drop,
        "metric":"live requested allocator bytes; not RSS; not a timing result"}));
    #[cfg(feature="images")] coexist();
}

#[cfg(feature="images")]
fn coexist() {
    use std::sync::Arc;
    use nebula_terminal::grid::GridCell;
    use nebula_terminal::inline_image::{ImageCell, ImageIdentity};
    let seed = format!("{}# Shared marker\r\n{}", fixture::START, fixture::DONE);
    let before = live();
    let mut term = Term::new(Config::default(), &Size, Lens);
    let mut stream = StreamProcessor::default();
    stream.feed(&mut term, &Lens, seed.as_bytes());
    let mut template = term.grid()[Line(0)][Column(0)].clone();
    let surface = Arc::downgrade(&template.inline_marker().unwrap().surface);
    let owner = ImageIdentity::new();
    let pixels = Arc::downgrade(&owner);
    template.set_image(ImageCell { image: owner.clone(), column: 0, row: 0 });
    let mut cells: Vec<_> = (0..9600).map(|n| {
        let mut cell = template.clone();
        cell.set_image(ImageCell { image: owner.clone(), column: n % 120, row: n / 120 });
        cell
    }).collect();
    drop(owner);
    assert!(cells.iter().all(|cell| cell.image().is_some() && cell.inline_marker().is_some()));
    let fill = live() - before;
    for cell in &mut cells { cell.discard(); }
    template.discard();
    assert!(pixels.upgrade().is_none());
    assert!(cells.iter().all(|cell| cell.extra.is_none()));
    let after_discard = live() - before;
    stream.feed(&mut term, &Lens, b"\x1bc");
    assert!(surface.upgrade().is_none());
    drop(cells); drop(template); drop(stream); drop(term); drop(surface); drop(pixels);
    let after_drop = live() - before;
    assert_eq!(after_drop, 0);
    println!("{}", serde_json::json!({"case":"9600_marker_and_image_cells",
        "fill":fill, "after_discard":after_discard, "after_drop":after_drop,
        "both_owners_released":true, "metric":"live requested allocator bytes; not RSS"}));
}
