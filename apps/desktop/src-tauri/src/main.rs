//! Native executable entry point.

// Prevents an additional console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = branlly_desktop_lib::run() {
        eprintln!("Branlly failed to start: {error}");
        std::process::exit(1);
    }
}
