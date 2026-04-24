// Windows subsystem opt-out so a Windows release build doesn't pop a
// console window. Ignored on Linux/macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vaner_linux_lib::run();
}
