#![windows_subsystem = "windows"]

extern crate select;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate hyper_tls;
extern crate cookie;
extern crate rand;

//required by gui
#[macro_use] extern crate conrod;
extern crate gfx_window_glutin;
extern crate find_folder;
extern crate image;
extern crate glutin;
extern crate winit;
extern crate gfx;
extern crate gfx_core;
extern crate rusttype;

extern crate clipboard;

mod igg;
mod megaup;
mod drive;
mod util;
mod gui;
mod support;

fn main() {
    gui::show();
}

