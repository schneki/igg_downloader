extern crate select;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate hyper_tls;
extern crate cookie;

extern crate gio;
extern crate glib;
extern crate gtk;

mod igg;
mod drive;
mod util;
mod gui;

fn main() {
    gui::show();
}

