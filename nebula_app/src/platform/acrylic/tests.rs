use super::composition::Context;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, PM_REMOVE, PeekMessageW, TranslateMessage,
    WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW,
};

struct TestWindow(isize);
impl TestWindow {
    fn new() -> Self {
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP,
                windows_core::w!("STATIC").as_ptr(),
                windows_core::w!("Pebrel Acrylic lifecycle test").as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                320,
                240,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        assert!(!hwnd.is_null());
        Self(hwnd as isize)
    }
}
impl Drop for TestWindow {
    fn drop(&mut self) {
        assert_ne!(unsafe { DestroyWindow(self.0 as _) }, 0);
    }
}

#[test]
#[ignore = "requires interactive Windows 11 and Windows App Runtime 1.8 >= 8000.946.1701.0"]
fn native_targets_rollback_switch_independently_and_drain_before_sta_exit() {
    // Real COM/WinRT calls protect ABI, failure rollback, two HWND ownership and
    // shutdown. This does not substitute for GPUI focus/edge screenshot checks.
    let mut context = Context::new().expect("required native runtime");
    let first = TestWindow::new();
    let second = TestWindow::new();
    assert!(context.attach(0).is_err());
    for _ in 0..8 {
        let a = context.attach(first.0).expect("first native target");
        let b = context.attach(second.0).expect("independent second target");
        assert!(a.has_live_full_size_root() && b.has_live_full_size_root());
        drop(a);
        assert!(b.has_live_full_size_root());
        // Reattach the same HWND while the other remains alive (material toggle).
        let a = context.attach(first.0).expect("reattach after Close");
        let mut message = unsafe { std::mem::zeroed() };
        unsafe {
            while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        drop(b);
        assert!(a.has_live_full_size_root());
        drop(a);
    }
    drop(first);
    drop(second);
    assert!(context.shutdown(), "owned queue must actually complete shutdown");
    assert!(windows::System::DispatcherQueue::GetForCurrentThread().is_err());
}
