// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate lazy_static;

mod ssh;

fn main() {
    // Workaround for IMKCFRunLoopWakeUpReliable error
    std::env::set_var("NSUnbufferedIO", "YES");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ssh::create_ssh_connection,
            ssh::send_ssh_data,
            ssh::read_ssh_data,
            ssh::resize_pty,
            ssh::close_ssh_connection  // Add this line
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
