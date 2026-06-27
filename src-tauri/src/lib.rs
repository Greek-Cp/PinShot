//! PinShot Tauri shell entry point.
//!
//! The shell owns the OS integration (tray, global hotkey, overlay windows, and
//! the platform capture/clipboard/file adapters). Domain logic lives in
//! `pinshot-core`; this layer wires it to Tauri commands. Kept thin on purpose
//! (Constitution IV).

mod capture;
mod editor;
mod external;

use tauri::Manager;

/// Builds and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Menu-bar/background app: no Dock icon and no main window on launch.
            // The hidden "main" window stays alive purely to anchor the app's
            // lifecycle; all interaction happens via the tray and global hotkey.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            capture::setup(app.handle())?;
            editor::setup(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            capture::get_overlay_frame,
            capture::commit_selection,
            capture::commit_annotated,
            capture::edit_selection,
            capture::copy_color,
            capture::cancel_capture,
            capture::create_pin,
            capture::pin_annotated,
            capture::get_pin_image,
            capture::close_pin,
            capture::raise_pin,
            capture::copy_pin,
            capture::save_pin,
            capture::read_image,
            capture::reveal_in_finder,
            capture::copy_path,
            capture::delete_file,
            capture::close_preview,
            capture::reposition_all_pins,
            editor::editor_get_image,
            editor::editor_get_doc,
            editor::editor_add,
            editor::editor_update,
            editor::editor_delete,
            editor::editor_undo,
            editor::editor_redo,
            editor::editor_clear,
            editor::editor_export,
            editor::editor_close,
            editor::editor_detect_qr,
            editor::editor_pick_color,
            editor::editor_crop,
            editor::copy_text,
            editor::open_external,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build PinShot");

    // Run with an event loop that repositions off-screen pins whenever the app
    // regains focus (covers the "display removed under a pin" edge case).
    app.run(|app_handle, event| {
        if let tauri::RunEvent::WindowEvent {
            event: tauri::WindowEvent::Focused(true),
            ..
        } = &event
        {
            let pins = app_handle.state::<capture::PinRegistry>();
            capture::pin::reposition_all_offscreen(app_handle, &pins);
        }
    });
}
