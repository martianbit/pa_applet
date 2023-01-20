use gtk::prelude::*;
use gtk::Menu;
use libappindicator::{AppIndicator, AppIndicatorStatus};

fn main() {
    gtk::init().unwrap();

    let mut indicator = AppIndicator::new("pa applet", "audio-volume-high");
    indicator.set_status(AppIndicatorStatus::Active);

    let mut menu = Menu::new();

    indicator.set_menu(&mut menu);
    menu.show_all();

    gtk::main();
}

