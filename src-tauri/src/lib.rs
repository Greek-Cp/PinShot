//! PinShot Tauri shell entry point.
//!
//! The shell owns the OS integration (tray, global hotkey, overlay windows, and
//! the platform capture/clipboard/file adapters). Domain logic lives in
//! `pinshot-core`; this layer wires it to Tauri commands. Kept thin on purpose
//! (Constitution IV).

mod capture;

/// Builds and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            capture::setup(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            capture::get_overlay_frame,
            capture::commit_selection,
            capture::copy_color,
            capture::cancel_capture,
            capture::create_pin,
            capture::get_pin_image,
            capture::close_pin,
            capture::raise_pin,
            capture::copy_pin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PinShot");
}
