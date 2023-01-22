use std::rc::Rc;
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

const DIALOG_WIDTH: u32 = 200;
const DIALOG_HEIGHT: u32 = 50;
const DIALOG_POS_X: u32 = 1920 - DIALOG_WIDTH - 10;
const DIALOG_POS_Y: u32 = 40;
const ICONS_NAME: &str = "Papirus-Dark";

static mut DIALOG: Option<Dialog> = None;

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

        widget.set_icon(&QIcon::from_theme_1a(&QString::from_std_str("audio-volume-high")));
        widget.show();

        let this = Rc::new(Self {
            widget,
        });

        this.init();
        this
    }

    unsafe fn init(self: &Rc<Self>) {
        self.widget.activated().connect(&self.slot_on_click());
    }

    #[slot(SlotNoArgs)]
    unsafe fn on_click(self: &Rc<Self>) {
        DIALOG.as_ref().unwrap().show_all();
    }
}

fn main() {
    gtk::init().unwrap();

    let dialog = Dialog::new();

    dialog.move_(DIALOG_POS_X as i32, DIALOG_POS_Y as i32);
    dialog.resize(DIALOG_WIDTH as i32, DIALOG_HEIGHT as i32);

    let scale = Scale::with_range(Orientation::Horizontal, 0_f64, 100_f64, 1_f64);
    dialog.content_area().pack_start(&scale, true, true, 0);

    dialog.connect_focus_out_event(|_, _| unsafe {
        DIALOG.as_ref().unwrap().hide();
        Inhibit(false)
    });

    unsafe {
        DIALOG = Some(dialog);
    }

    QApplication::init(|_| unsafe {
        QIcon::set_theme_name(&QString::from_std_str(ICONS_NAME));
        let _tray_icon = TrayIcon::new();
        QApplication::exec()
    });
}

