mod api;
mod app;
mod auth;
mod components;
mod models;
mod pages;

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(app::App);
}