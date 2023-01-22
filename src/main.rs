use std::rc::Rc;
use std::process::Command;
use cpp_core::Ptr;
use cpp_core::StaticUpcast;
use gtk::prelude::*;
use gtk::Dialog;
use gtk::Scale;
use gtk::Orientation;
use gtk::Inhibit;
use qt_core::QString;
use qt_core::QObject;
use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::slot;
use qt_gui::QIcon;
use qt_widgets::QApplication;
use qt_widgets::QSystemTrayIcon;

const MAX_ABS_VOLUME: u32 = 65537;
const DIALOG_WIDTH: u32 = 200;
const DIALOG_HEIGHT: u32 = 50;
const DIALOG_POS_X: u32 = 1920 - DIALOG_WIDTH - 10;
const DIALOG_POS_Y: u32 = 40;
const ICONS_NAME: &str = "Papirus-Dark";

static mut TRAY_ICON: Option<Rc<TrayIcon>> = None;

fn set_volume(volume: u8) {
    let abs_volume = (volume as u32) * MAX_ABS_VOLUME / 100;
    Command::new("pactl").args(["set-sink-volume", "@DEFAULT_SINK@", &abs_volume.to_string()]).spawn().unwrap();
}

fn get_current_volume() -> u8 {
    let abs_volume_raw = String::from_utf8(Command::new("pactl").args(["get-sink-volume", "@DEFAULT_SINK@"]).output().unwrap().stdout).unwrap();
    let abs_volume: u32 = abs_volume_raw.split_whitespace().nth(2).unwrap().parse().unwrap();

    ((abs_volume as f64) * 100_f64 / (MAX_ABS_VOLUME as f64)).ceil() as u8
}

fn get_correct_icon_name(volume: u8) -> &'static str {
    if volume == 0 { "audio-volume-muted" }
    else if volume <= 33 { "audio-volume-low" }
    else if volume <= 66 { "audio-volume-medium" }
    else { "audio-volume-high" }
}

struct TrayIcon {
    widget: QBox<QSystemTrayIcon>
}

impl StaticUpcast<QObject> for TrayIcon {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.widget.as_ptr().static_upcast()
    }
}

impl TrayIcon {
    unsafe fn new() -> Rc<Self> {
        let widget = QSystemTrayIcon::new();

        let this = Rc::new(Self {
            widget,
        });

        this.init();
        this
    }

    unsafe fn init(self: &Rc<Self>) {
        self.widget.activated().connect(&self.slot_on_click());
        self.update_icon();

        self.widget.show();
    }

    unsafe fn update_icon(self: &Rc<Self>) {
        let icon_name = get_correct_icon_name(get_current_volume());
        self.widget.set_icon(&QIcon::from_theme_1a(&QString::from_std_str(icon_name)));
    }

    #[slot(SlotNoArgs)]
    fn on_click(self: &Rc<Self>) {
        let scale = Scale::with_range(Orientation::Horizontal, 0_f64, 100_f64, 1_f64);

        scale.set_value(get_current_volume() as f64);
        scale.set_draw_value(false);

        scale.connect_value_changed(|x| unsafe {
            set_volume(x.value() as u8);
            TRAY_ICON.as_ref().unwrap().update_icon();
        });

        let dialog = Dialog::new();

        dialog.move_(DIALOG_POS_X as i32, DIALOG_POS_Y as i32);
        dialog.resize(DIALOG_WIDTH as i32, DIALOG_HEIGHT as i32);

        dialog.content_area().pack_start(&scale, true, true, 0);

        dialog.connect_focus_out_event(|x, _| {
            x.emit_close();
            Inhibit(true)
        });

        dialog.show_all();
    }
}

fn main() {
    gtk::init().unwrap();

    QApplication::init(|_| unsafe {
        QIcon::set_theme_name(&QString::from_std_str(ICONS_NAME));
        TRAY_ICON = Some(TrayIcon::new());
        QApplication::exec()
    });
}

