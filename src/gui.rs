use gio;
use glib;
use gtk;

use gio::{ApplicationExt, ApplicationExtManual};
use gtk::{TextBufferExt, WidgetExt, ContainerExt, ButtonExt, EntryExt, GtkWindowExt, Inhibit, TextViewExt};

use std::cell::RefCell;
use std::env::args;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("IGG Downloader");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(600, 400);

    window.connect_delete_event(clone!(window => move |_, _| {
        window.destroy();
        Inhibit(false)
    }));

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let game_url_entry = gtk::Entry::new();
    let download_button = gtk::Button::new_with_label("Download");
    let status_text_view = gtk::TextView::new();
    
    container.add(&game_url_entry);
    container.add(&download_button);
    container.add(&status_text_view);



    let (tx_origin, rx) = channel();
    // put TextBuffer and receiver in thread local storage
    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((status_text_view.get_buffer()
                                              .expect("Couldn't get buffer from text_view"),
                                     rx))
    });

    let tx_clone = tx_origin.clone();

    download_button.connect_clicked(move |_| {
        let tx = tx_clone.clone(); 
        let game_url = game_url_entry.clone().get_text().unwrap();
        thread::spawn(move|| {
            tx.send("Downloading...".into()).unwrap();
            
            match ::igg::download_game(&game_url, tx.clone()) {
                Ok(_) => {
                    tx.send("Download finished".into()).unwrap();
                },
                Err(err) => {
                    tx.send(err).unwrap();
                }
            };


        });
    });

    thread::spawn(move || {
        loop {
            glib::idle_add(receive);
            thread::sleep(Duration::from_millis(32));
        }
    });


    window.add(&container);
    window.show_all();
}

pub fn receive() -> glib::Continue {
    GLOBAL.with(|global| {
        if let Some((ref buf, ref rx)) = *global.borrow() {
            if let Ok(text) = rx.try_recv() {
                buf.set_text(&text);
            }
        }
    });
    glib::Continue(false)
}

// declare a new thread local storage key
thread_local!(
    static GLOBAL: RefCell<Option<(gtk::TextBuffer, Receiver<String>)>> = RefCell::new(None)
);

pub fn show() {
    let application = gtk::Application::new("com.github.multithreading_context",
                                            gio::ApplicationFlags::empty())
                                       .expect("Initialization failed...");

    application.connect_startup(move |app| {
        build_ui(app);
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
