use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize,Ordering};
use std::time::Instant;
use nebula_terminal::event::{VoidListener,WindowSize};
use nebula_terminal::event_loop::StreamProcessor;
use nebula_terminal::grid::Dimensions;
use nebula_terminal::index::{Column,Line};
use nebula_terminal::term::{Config,Term};
use nebula_terminal::term::cell::{Cell,CellExtra};

static LIVE:AtomicIsize=AtomicIsize::new(0);
struct Counted;
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::Relaxed);}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){LIVE.fetch_sub(l.size() as isize,Ordering::Relaxed);unsafe{System.dealloc(p,l)};}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {let q=unsafe{System.realloc(p,l,n)};if !q.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::Relaxed);}q}
}
#[global_allocator]static ALLOC:Counted=Counted;
fn live()->isize{LIVE.load(Ordering::Relaxed)}
const COLS:usize=120;const ROWS:usize=40;const HISTORY:usize=10_000;const INPUT_ROWS:usize=12_000;
struct Size;
impl Dimensions for Size{fn total_lines(&self)->usize{ROWS}fn screen_lines(&self)->usize{ROWS}fn columns(&self)->usize{COLS}}
fn payload(case:&str)->Vec<u8>{
    let mut bytes=Vec::new();
    if case.starts_with("images") {
        for n in 0..40 {
            if case=="images_linked"{bytes.extend_from_slice(format!("\x1b]8;id={n};https://example.test/{n}\x07\x1b[58;2;1;2;3m").as_bytes());}
            bytes.extend_from_slice(b"\x1b]1337;File=inline=1;width=120;height=80;preserveAspectRatio=0:iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLttAAAAABJRU5ErkJggg==\x07\r\n");
        }
    } else {
        for n in 0..INPUT_ROWS {
            match case {
                "underline_rows"=>bytes.extend_from_slice(format!("\x1b[58;2;{};2;3m",n%256).as_bytes()),
                "hyperlink_rows"=>bytes.extend_from_slice(format!("\x1b]8;id={n};https://example.test/{n}\x07").as_bytes()),
                _=>{},
            }
            for col in 0..COLS {
                if case=="underline_cells"{bytes.extend_from_slice(format!("\x1b[58;2;{};2;3m",col%256).as_bytes());}
                bytes.extend_from_slice(if case=="combining"{"e\u{301}".as_bytes()}else{b"x"});
            }
            bytes.extend_from_slice(b"\r\n");
        }
    }
    bytes.extend_from_slice(b"\x1b]8;;\x07\x1b[0m");bytes
}
fn run(case:&str){
    let payload=payload(case);
    let before=live();
    let mut term=Term::new(Config{scrolling_history:HISTORY,..Config::default()},&Size,VoidListener);
    let mut stream=StreamProcessor::default();
    stream.resize(WindowSize{num_cols:COLS as u16,num_lines:ROWS as u16,cell_width:10,cell_height:20});
    let start=Instant::now();
    for chunk in payload.chunks(64*1024){stream.feed(&mut term,&VoidListener,chunk);}
    let feed_us=start.elapsed().as_micros();
    let fill=live()-before;
    let mut checksum=0u64;let mut tiles=0usize;
    for row in -(term.grid().history_size() as i32)..ROWS as i32{
        for col in 0..COLS{
            let cell=&term.grid()[Line(row)][Column(col)];
            checksum=checksum.wrapping_add(cell.c as u64);
            for c in cell.zerowidth().unwrap_or_default(){checksum=checksum.wrapping_add(*c as u64);}
            checksum=checksum.wrapping_add(cell.underline_color().is_some() as u64);
            if let Some(link)=cell.hyperlink(){checksum=checksum.wrapping_add(link.uri().len() as u64);}
            #[cfg(feature="images")]
            if cell.image().is_some(){tiles+=1;}
        }
    }
    let history=term.grid().history_size();
    stream.feed(&mut term,&VoidListener,b"\x1b[3J");let clear=live()-before;
    stream.feed(&mut term,&VoidListener,b"\x1bc");let reset=live()-before;
    term.grid_mut().truncate();let truncate=live()-before;
    drop(stream);drop(term);let after_drop=live()-before;
    assert_eq!(after_drop,0,"{case}");
    println!("case={case} payload={} cell={} extra={} history={history} tiles={tiles} checksum={checksum} fill={fill} clear_history={clear} reset={reset} truncate={truncate} after_drop={after_drop} instrumented_feed_us={feed_us}",payload.len(),std::mem::size_of::<Cell>(),std::mem::size_of::<CellExtra>());
}
fn main(){
    println!("Windows x64; requested live allocation bytes, not RSS; timing includes allocation instrumentation; 120x40 grid, 10000 history, 12000 text rows or 40 images of 120x80 cells.");
    for case in ["plain","combining","underline_rows","underline_cells","hyperlink_rows"]{run(case);}
    #[cfg(feature="images")]
    for case in ["images","images_linked"]{run(case);}
}
