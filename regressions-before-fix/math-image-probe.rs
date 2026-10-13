    #[cfg(feature = "gpui-test-support")]
    #[gpui::test]
    fn review_protocol_image_with_real_math_bitmaps_keeps_text_visible(cx: &mut gpui::TestAppContext) {
        use base64::Engine as _;
        use std::io::Write as _;
        use nebula_terminal::event::{Event, EventListener, WindowSize};
        use nebula_terminal::event_loop::StreamProcessor;
        use nebula_terminal::render::{RenderSnapshot, SnapshotConfig};
        use std::sync::{Arc, Mutex};
        use super::super::inline_image::{InlineImageStore, decode};

        #[derive(Clone, Default)]
        struct Listener(Arc<Mutex<Vec<Event>>>);
        impl EventListener for Listener {
            fn send_event(&self, event: Event) { self.0.lock().unwrap().push(event); }
        }
        cx.update(crate::gpui_shell::math_view::register);
        let pixels = image::RgbaImage::from_pixel(4, 1, image::Rgba([240, 40, 40, 255]));
        let mut output = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(pixels).write_to(&mut output, image::ImageFormat::Png).unwrap();
        let png = base64::engine::general_purpose::STANDARD.encode(output.into_inner());
        let protocol = format!("\x1b]1337;File=inline=1;width=4;height=1;preserveAspectRatio=0:{png}\x07");
        let cases = [
            ("main", "", "$x^2$ ", "TAIL\r\nprompt", true),
            ("alternate", "\x1b[?1049h", "$x^2$ ", "TAIL\r\nprompt", true),
            ("active-line", "", "$x^2$ ", "TAIL", false),
            ("ansi-code-background", "", "\x1b[44m$x^2$\x1b[49m ", "TAIL\r\nprompt", false),
        ];
        let mut failures = Vec::new();
        for (label, screen, prefix, suffix, expect_formula) in cases {
            let listener = Listener::default();
            let mut term = Term::new(Config::default(), &TestSize { cols: 20, rows: 8 }, listener.clone());
            let mut stream = StreamProcessor::default();
            stream.resize(WindowSize { num_cols:20, num_lines:8, cell_width:8, cell_height:18 });
            stream.feed(&mut term, &listener, format!("{screen}{prefix}{protocol}{suffix}").as_bytes());
            let mut store = InlineImageStore::default();
            for event in listener.0.lock().unwrap().drain(..) {
                if let Event::InlineImage { data, placement } = event { store.enqueue(data, placement).unwrap(); }
            }
            while let Some(pending) = store.start_next() { store.finish(decode(pending)).unwrap(); }
            let images = store.frame_images();
            assert_eq!(images.len(), 1, "actual production image decode: {label}");
            let snap = RenderSnapshot::capture(&term, &SnapshotConfig { rows:8, cols:20 });
            assert_eq!(snap.image_runs.len(),1);
            let run = &snap.image_runs[0];
            assert_eq!((run.row,run.column,run.columns),(0,6,4));
            let tail: Vec<_> = snap.segments.iter().filter(|segment|segment.row==0)
                .flat_map(|segment|segment.cells.iter()).filter(|cell|cell.col>=10 && cell.col<14)
                .map(|cell|(cell.col,cell.text.clone())).collect();
            assert_eq!(tail.iter().map(|(_,text)|text.as_str()).collect::<String>(),"TAIL");
            let mut overlay = MathOverlay::default();
            cx.update(|cx| overlay.bind_resources(cx));
            let size = grid_size_info(20,8,8.0,18.0);
            let mut counts = Vec::new();
            let mut frame = super::MathFrame::default();
            for _ in 0..12 {
                frame = cx.update(|cx| {
                    let pending = overlay.plan_frame(&term,&size,Rgb::new(0xec,0xef,0xf4),16.0,1.0);
                    overlay.finalize_frame(pending,1.0,cx)
                });
                counts.push(frame.formulas.len());
                if !expect_formula || frame.formulas.len()==1 { break; }
                cx.run_until_parked();
            }
            assert_eq!(frame.formulas.len(),usize::from(expect_formula),"real bitmap pipeline {label}: {counts:?}");
            let image_left = 8.0*run.column as f32;
            let image_right = image_left+8.0*(images[0].placement.layout.width/images[0].placement.layout.cell_width);
            let projected: Vec<_> = tail.iter().map(|(column,text)| {
                let visual=frame.project_cell(0,*column as usize,20).unwrap();
                (text.clone(),*column,visual,8.0*visual as f32)
            }).collect();
            let overlap: Vec<_> = projected.iter().filter(|(_,_,_,left)|*left < image_right && *left+8.0 > image_left).collect();
            writeln!(std::io::stdout().lock(), "CASE={label} formulas={} frame_counts={counts:?} image_px=[{image_left},{image_right}) tail_source_visual_px={projected:?} overlapping_cells={overlap:?}",frame.formulas.len()).unwrap();
            if !overlap.is_empty() { failures.push(label); }
        }
        assert!(failures.is_empty(),"production image paint overlaps projected text in {failures:?}");
    }
