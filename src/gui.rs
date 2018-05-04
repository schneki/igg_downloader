
use gtk;
use gtk::{Label, Entry, Button, Window, WindowType};
use gtk::{LabelExt, ButtonExt, WidgetExt, GtkWindowExt, ContainerExt, BoxExt, EntryExt};
use gtk::Inhibit;


pub fn show() {    
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("IGG Downloader");
    window.set_default_size(350, 70);
    let layout = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let label = Label::new("Status (wait for 'Finished Download' message to appear before closing)");
    let button = Button::new_with_label("Download");
    let entry = Entry::new();

    layout.pack_start(&entry, true, true, 0);
    layout.pack_start(&button, true, true, 0);
    layout.pack_start(&label, true, true, 0);
    window.add(&layout);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    button.connect_clicked(move |_| {
        let url = entry.get_text().unwrap();
        println!("{}", url);

        let drive_urls = ::igg::get_drive_urls(&url);
        for u in drive_urls {
            ::drive::download(&u)
        }

        label.set_text("Finished Download");
    });

    gtk::main();
}
