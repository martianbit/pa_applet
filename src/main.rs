use std::rc::Rc;
use cpp_core::Ptr;
use cpp_core::CppBox;
use cpp_core::StaticUpcast;
use qt_core::QString;
use qt_core::QObject;
use qt_core::QPoint;
use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::slot;
use qt_gui::QCursor;
use qt_gui::QIcon;
use qt_widgets::QApplication;
use qt_widgets::QSystemTrayIcon;

const ICONS_NAME: &str = "Papirus-Dark";

struct PaApplet {
    tray_icon: QBox<QSystemTrayIcon>
}

impl StaticUpcast<QObject> for PaApplet {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.tray_icon.as_ptr().static_upcast()
    }
}

impl PaApplet {
    unsafe fn new() -> Rc<Self> {
        let tray_icon = QSystemTrayIcon::new();

        tray_icon.set_icon(&QIcon::from_theme_1a(&QString::from_std_str("audio-volume-high")));
        tray_icon.show();

        let this = Rc::new(Self {
            tray_icon
        });

        this.init();
        this
    }

    unsafe fn init(self: &Rc<Self>) {
        self.tray_icon.activated().connect(&self.slot_on_click());
    }

    #[slot(SlotNoArgs)]
    unsafe fn on_click(self: &Rc<Self>) {
        let point: CppBox<QPoint> = QCursor::pos_0a();
        println!("{} {}", point.x(), point.y());
    }
}

fn main() {
    QApplication::init(|_| unsafe {
        QIcon::set_theme_name(&QString::from_std_str(ICONS_NAME));
        let _pa_applet = PaApplet::new();
        QApplication::exec()
    });
}

