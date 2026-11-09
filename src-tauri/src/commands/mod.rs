pub mod auth;
pub mod profile;

use rtube_macros::app_command;

#[app_command]
pub fn greet(name: String) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
