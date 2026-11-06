//! Native work area is queried once at startup; rendering remains entirely on CPU.
pub fn fit_work_area(width: usize, height: usize) -> (usize, usize) {
    (
        width.saturating_sub(64).clamp(1, 1100),
        height.saturating_sub(100).clamp(1, 700),
    )
}
#[cfg(target_os = "windows")]
pub fn initial_size() -> (usize, usize) {
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn SystemParametersInfoW(action: u32, param: u32, data: *mut Rect, flags: u32) -> i32;
    }
    let mut area = Rect {
        left: 0,
        top: 0,
        right: 1024,
        bottom: 768,
    };
    // SPI_GETWORKAREA excludes the taskbar; leave additional room for borders and caption.
    if unsafe { SystemParametersInfoW(0x0030, 0, &mut area, 0) } == 0 {
        return (800, 600);
    }
    fit_work_area(
        (area.right - area.left).max(1) as usize,
        (area.bottom - area.top).max(1) as usize,
    )
}
#[cfg(not(target_os = "windows"))]
pub fn initial_size() -> (usize, usize) {
    (800, 600)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inicio_cabe_en_area_de_trabajo() {
        for (w, h) in [(1920, 1040), (1366, 728), (800, 560)] {
            let (x, y) = fit_work_area(w, h);
            assert!(x + 64 <= w && y + 100 <= h);
        }
    }
}

#[cfg(all(test, target_os = "windows"))]
mod desktop_tests {
    use crate::{framebuffer::Framebuffer, presentation, screens, ui};
    use minifb::{Window, WindowOptions};
    use std::ffi::c_void;
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn ShowWindow(window: *mut c_void, command: i32) -> i32;
        fn GetWindowLongW(window: *mut c_void, index: i32) -> i32;
        fn AdjustWindowRectEx(rect: *mut Rect, style: u32, menu: i32, extended: u32) -> i32;
        fn SetWindowPos(
            window: *mut c_void,
            after: *mut c_void,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            flags: u32,
        ) -> i32;
    }
    fn pump(window: &mut Window) {
        for _ in 0..4 {
            window.update();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
    #[test]
    #[ignore = "Abre una ventana real para verificar resize, maximizar, minimizar y restaurar"]
    fn ventana_real_redimensiona_y_restaura() {
        let mut window = Window::new(
            "The Showgirl Universe - prueba de tamanos",
            800,
            600,
            WindowOptions {
                resize: true,
                ..WindowOptions::default()
            },
        )
        .unwrap();
        let handle = window.get_window_handle();
        let mut buffer = Framebuffer::new(800, 600);
        for (width, height) in [(800, 600), (1000, 650), (1280, 720)] {
            let mut rect = Rect {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            unsafe {
                assert_ne!(
                    AdjustWindowRectEx(
                        &mut rect,
                        GetWindowLongW(handle, -16) as u32,
                        0,
                        GetWindowLongW(handle, -20) as u32
                    ),
                    0
                );
                assert_ne!(
                    SetWindowPos(
                        handle,
                        std::ptr::null_mut(),
                        0,
                        0,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        0x0002 | 0x0004
                    ),
                    0
                );
            }
            pump(&mut window);
            let actual = window.get_size();
            // Windows can clamp a requested size to the monitor work area.
            assert!(actual.0 > 0 && actual.1 > 0 && actual.0 <= width && actual.1 <= height);
            buffer.resize(actual.0, actual.1);
            let (client_width, client_height) = actual;
            for screen in 0..3 {
                buffer.clear(0x101522);
                match screen {
                    0 => screens::menu(&mut buffer, 0, true),
                    1 => screens::gallery_ui(
                        &mut buffer,
                        &screens::gallery_layout(client_width, client_height, 4, true),
                    ),
                    _ => {
                        presentation::hud(
                            &mut buffer,
                            "The Life of a Showgirl",
                            12.0,
                            "LIBRE | C: RODEAR PLANETA",
                            true,
                        );
                    }
                }
                ui::footer(&mut buffer, "AUDIO: LISTO | M: SILENCIO | -/+: VOLUMEN");
                window
                    .update_with_buffer(&buffer.pixels, buffer.width, buffer.height)
                    .unwrap();
            }
            println!(
                "Solicitada {width}x{height}; ventana real y presentacion: {}x{} OK",
                actual.0, actual.1
            );
        }
        unsafe {
            ShowWindow(handle, 3);
        }
        pump(&mut window);
        let maximized = window.get_size();
        assert!(maximized.0 > 0 && maximized.1 > 0);
        buffer.resize(maximized.0, maximized.1);
        window
            .update_with_buffer(&buffer.pixels, buffer.width, buffer.height)
            .unwrap();
        println!("Maximizada: {}x{} OK", maximized.0, maximized.1);
        unsafe {
            ShowWindow(handle, 6);
        }
        pump(&mut window);
        assert_eq!(window.get_size(), (0, 0));
        let retained = buffer.pixels.len();
        assert!(!buffer.resize(0, 0));
        assert_eq!(retained, buffer.pixels.len());
        unsafe {
            ShowWindow(handle, 9);
        }
        pump(&mut window);
        let restored = window.get_size();
        assert!(restored.0 > 0 && restored.1 > 0);
        buffer.resize(restored.0, restored.1);
        window
            .update_with_buffer(&buffer.pixels, buffer.width, buffer.height)
            .unwrap();
        println!("Minimizar y restaurar: OK");
    }
}
