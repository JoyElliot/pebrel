use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};
#[cfg(feature="images")]
use nebula_terminal::grid::GridCell;
use nebula_terminal::term::cell::{Cell, Hyperlink};
use nebula_terminal::vte::ansi::{Color, Rgb};

static LIVE: AtomicIsize = AtomicIsize::new(0);
struct Counted;
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() { LIVE.fetch_add(layout.size() as isize, Ordering::Relaxed); }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        unsafe { System.dealloc(p, layout) };
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, n: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, layout, n) };
        if !q.is_null() { LIVE.fetch_add(n as isize - layout.size() as isize, Ordering::Relaxed); }
        q
    }
}
#[global_allocator] static ALLOC: Counted = Counted;
fn live() -> isize { LIVE.load(Ordering::Relaxed) }

fn cells_case(name: &str, make: impl Fn(usize) -> Cell) {
    const N: usize = 100_000;
    let before = live();
    let mut cells: Vec<Cell> = (0..N).map(make).collect();
    let after_fill = live() - before;
    #[cfg(feature="images")]
    for cell in &mut cells { cell.discard(); }
    let after_discard = live() - before;
    let with_extra = cells.iter().filter(|cell| cell.extra.is_some()).count();
    drop(cells);
    println!("case={name} count={N} requested_live_fill={after_fill} requested_live_after_discard={after_discard} retained_extras={with_extra} after_drop={}", live()-before);
}

#[cfg(feature="images")]
fn grid_case(with_image: bool) {
    use nebula_terminal::grid::Grid;
    use nebula_terminal::index::{Column, Line};
    use nebula_terminal::inline_image::{ImageCell, ImageIdentity};
    use std::sync::Arc;
    let before = live();
    let mut grid = Grid::<Cell>::new(1, 1, 1);
    let owner = ImageIdentity::new();
    let weak = Arc::downgrade(&owner);
    if with_image {
        grid[Line(0)][Column(0)].set_image(ImageCell { image: owner.clone(), column: 0, row: 0 });
    }
    drop(owner);
    grid.scroll_up::<Color>(&(Line(0)..Line(1)), 1);
    if with_image { assert!(weak.strong_count() > 0); }
    let in_history = live() - before;
    grid.scroll_up::<Color>(&(Line(0)..Line(1)), 1);
    assert_eq!(weak.strong_count(), 0);
    let after_eviction = live() - before;
    grid.clear_history();
    let after_clear = live() - before;
    grid.truncate();
    let after_truncate = live() - before;
    drop(grid);
    drop(weak);
    let after_drop = live() - before;
    println!("grid_image={with_image} in_history={in_history} after_eviction={after_eviction} after_clear={after_clear} after_truncate={after_truncate} after_drop={after_drop}");
    assert_eq!(after_drop, 0);
}

fn main() {
    println!("Counts are live requested allocator bytes, not RSS or allocator usable size.");
    cells_case("plain", |_| Cell::default());
    cells_case("combining_unique", |_| {
        let mut c=Cell::default(); c.c='e'; c.push_zerowidth('\u{301}'); c
    });
    cells_case("underline_unique", |n| {
        let mut c=Cell::default(); c.set_underline_color(Some(Color::Spec(Rgb{r:n as u8,g:2,b:3}))); c
    });
    let mut template=Cell::default();
    template.set_hyperlink(Some(Hyperlink::new(Some("shared"), "https://example.test".into())));
    cells_case("hyperlink_shared_template", |_| template.clone());
    drop(template);
    cells_case("hyperlink_unique", |n| {
        let mut c=Cell::default(); c.set_hyperlink(Some(Hyperlink::new(Some(n), "https://example.test".into()))); c
    });
    #[cfg(feature="images")]
    {
        use nebula_terminal::inline_image::{ImageCell, ImageIdentity};
        let owner=ImageIdentity::new();
        cells_case("image_only", |n| {
            let mut c=Cell::default(); c.set_image(ImageCell{image:owner.clone(),column:n%120,row:n/120}); c
        });
        let mut image_template=Cell::default();
        image_template.set_underline_color(Some(Color::Spec(Rgb{r:1,g:2,b:3})));
        image_template.set_hyperlink(Some(Hyperlink::new(Some("image-style"),"https://example.test".into())));
        image_template.set_image(ImageCell{image:owner.clone(),column:0,row:0});
        cells_case("image_shared_text_template", |n| {
            let mut c=image_template.clone(); c.set_image(ImageCell{image:owner.clone(),column:n%120,row:n/120}); c
        });
        drop(image_template);
        cells_case("image_unique_combining", |n| {
            let mut c=Cell::default(); c.c='e'; c.push_zerowidth('\u{301}');
            c.set_image(ImageCell{image:owner.clone(),column:n%120,row:n/120}); c
        });
        assert_eq!(std::sync::Arc::strong_count(&owner),1);
        grid_case(false);
        grid_case(true);
    }
}
