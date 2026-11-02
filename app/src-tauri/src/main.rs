// Verhindert unter Windows ein zusätzliches Konsolenfenster im Release-Build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    archiver_app_lib::bridge::migrate_app_dir();
    // Von Chrome gestartet: Brücke zur Extension statt Fenster.
    if archiver_app_lib::bridge::is_host_invocation() {
        archiver_app_lib::bridge::run_host();
        return;
    }
    archiver_app_lib::run()
}
