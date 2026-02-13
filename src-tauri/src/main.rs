// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() {
    let args = app_lib::cli::CliArgs::parse();

    if args.mode.is_some() {
        // CLI mode: run without GUI
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            app_lib::cli::run_cli(args).await;
        });
    } else {
        // GUI mode: launch Tauri window
        app_lib::run();
    }
}
