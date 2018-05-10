use std;

use conrod;
use glutin;
use gfx;
use gfx_core;
use support;
use winit;
use image;
use find_folder;
use gfx_window_glutin;
use rusttype::Font;

use glutin::GlContext;
use gfx::Device;

use conrod::{widget, Widget, Positionable, Labelable, Sizeable};

use std::sync::mpsc::channel;

const WIN_W: u32 = 600;
const WIN_H: u32 = 420;
const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

const MARGIN: conrod::Scalar = 30.0;
const SHAPE_GAP: conrod::Scalar = 50.0;
const TITLE_SIZE: conrod::FontSize = 42;
const SUBTITLE_SIZE: conrod::FontSize = 32;

type DepthFormat = gfx::format::DepthStencil;

widget_ids! {
    pub struct Ids {
        canvas,
        title,
        label_game_url,
        text_edit_game_url,
        status,
        label_status,
        button_download,
        progress,

    }
}

use std::sync::{Arc, Mutex};

pub enum Status {
    InvalidURL,
    Value(usize, usize),
    Progress(usize, usize),
    Finished,
}

pub struct App {
    game_url: String,
    status: String,
    event_counter: u32,
    progress: String,
    locked: bool
}

impl App {
    pub fn new() -> App {
        App { progress: String::new(), game_url: String::new(), status: String::new(), event_counter: 0, locked: false }
    }
}

use std::sync::mpsc::{Receiver, Sender};

use self::Status::*;

fn gui_always_active(ui: &mut conrod::UiCell, ids: &Ids, app: &mut App, tx: Arc<Mutex<Sender<Status>>>, rx: &Receiver<Status>){
    match rx.try_recv() {
        Ok(status) => {
            match status {
                InvalidURL => { 
                    app.status = "Not a valid igg-games.com url".into(); 
                    app.locked = false; 
                    app.game_url = "".into();
                },
                Value(i, size) => { app.status = format!("Downloading part {} of {}", i+1, size) },
                Finished => { 
                    app.status = "Download finished".into(); 
                    app.locked = false; 
                    app.progress = "".into(); 
                    app.game_url = "".into();
                },
                Progress(p, total) => { 
                    app.progress = format!("Downloaded {} of {} Mb {}%", p, total, ((p as f32/total as f32)*100.0) as usize); 
                }, 
            }
        }
        Err(_) => {},
    };
    widget::Text::new(&app.status)
        .font_size(14)
        .mid_top_of(ids.canvas)
        .down_from(ids.label_status, 10.0) 
        .set(ids.status, ui);

    widget::Text::new(&app.progress)
        .font_size(14)
        .mid_top_of(ids.canvas)
        .down_from(ids.status, 10.0) 
        .set(ids.progress, ui);
    
    let side = 130.0;

    for _press in widget::Button::new()
        .label("Download")
        .mid_left_with_margin_on(ids.canvas, MARGIN)
        .down_from(ids.text_edit_game_url, 60.0)
        .w_h(side, side)
        .set(ids.button_download, ui)
    {
        if !app.locked {
            let game_url = app.game_url.clone();
            let tx_clone = tx.clone();
            ::std::thread::spawn(move || {
                let tx_mutex = tx_clone.clone();
                let tx = tx_mutex.lock().unwrap();
                println!("down from thread");
                ::igg::download_game(&game_url, &tx);
            });
            app.locked = true;
        }
    }

}

fn gui(ui: &mut conrod::UiCell, ids: &Ids, app: &mut App) { 
    widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);
    
    widget::Text::new("IGG Downloader").font_size(22)
        .mid_top_of(ids.canvas)
        .set(ids.title, ui);

    widget::Text::new("Status")
        .font_size(14)
        .mid_top_of(ids.canvas)
        .down_from(ids.title, 20.0) 
        .set(ids.label_status, ui);
    
    
    widget::Text::new("Enter igg-games.com Link below")
        .font_size(14)
        .mid_top_of(ids.canvas)
        .down_from(ids.progress, 20.0) 
        .set(ids.label_game_url, ui);

    app.event_counter += 1;

    for _edit in widget::TextEdit::new(&mut app.game_url)
        .font_size(12)
        .mid_top_of(ids.canvas)
        .down_from(ids.label_game_url, 10.0)
        .set(ids.text_edit_game_url, ui)
    {
        println!("happens");
        app.game_url = _edit;
    }

}

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

pub fn show() {

    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

    // Builder for window
    let builder = glutin::WindowBuilder::new()
        .with_title("IGG Downloader")
        .with_dimensions(WIN_W, WIN_H);

    let context = glutin::ContextBuilder::new()
        .with_multisampling(4);

    let mut events_loop = winit::EventsLoop::new();

    // Initialize gfx things
    let (window, mut device, mut factory, rtv, _) =
        gfx_window_glutin::init::<conrod::backend::gfx::ColorFormat, DepthFormat>(builder, context, &events_loop );
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut renderer = conrod::backend::gfx::Renderer::new(&mut factory, &rtv, window.hidpi_factor() as f64).unwrap();

    // Create Ui and Ids of widgets to instantiate
    let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();
    let ids = Ids::new(ui.widget_id_generator());

    // Load font from file
    let font_buffer = include_bytes!("../assets/fonts/NotoSans/NotoSans-Regular.ttf").to_vec();
    ui.fonts.insert(Font::from_bytes(font_buffer).unwrap());

    let mut image_map = conrod::image::Map::new();
    let mut app = App::new();
    let chann = channel();
    let tx = Arc::new(Mutex::new(chann.0));
    let rx = chann.1;

    'main: loop {
        ::std::thread::sleep(::std::time::Duration::from_millis(32));
        // If the window is closed, this will be None for one tick, so to avoid panicking with
        // unwrap, instead break the loop
        let (win_w, win_h) = match window.get_inner_size() {
            Some(s) => s,
            None => break 'main,
        };

        let dpi_factor = window.hidpi_factor();

        if let Some(primitives) = ui.draw_if_changed() {
            let dims = (win_w as f32 * dpi_factor, win_h as f32 * dpi_factor);

            //Clear the window
            encoder.clear(&rtv, CLEAR_COLOR);

            renderer.fill(&mut encoder,dims,primitives,&image_map);

            renderer.draw(&mut factory,&mut encoder,&image_map);

            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
        }

        let mut should_quit = false;
        events_loop.poll_events(|event|{
            let (w, h) = (win_w as conrod::Scalar, win_h as conrod::Scalar);
            let dpi_factor = dpi_factor as conrod::Scalar;

            // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), window.window()) {
                ui.handle_event(event);
            }

            // Close window if the escape key or the exit button is pressed
            match event {
                winit::Event::WindowEvent{event: winit::WindowEvent::KeyboardInput{input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::Escape),..}, ..}, .. } |
                winit::Event::WindowEvent{event: winit::WindowEvent::Closed, ..} =>
                    should_quit = true,
                winit::Event::WindowEvent{event: winit::WindowEvent::KeyboardInput{input: winit::KeyboardInput{
                    virtual_keycode: Some(winit::VirtualKeyCode::V), modifiers: winit::ModifiersState{shift:false, ctrl: true, alt: false, logo: false}, ..
                },..},..} =>
                {
                    app.game_url = ctx.get_contents().unwrap();

                },
                _ => {},
            }
        });
        if should_quit {
            break 'main;
        }

        // Update widgets if any event has happened
        let mut ui_cell = ui.set_widgets();
        gui(&mut ui_cell, &ids, &mut app);
        gui_always_active(&mut ui_cell, &ids, &mut app, tx.clone(), &rx);
        //let mut ui = ui.set_widgets();
    }
}
