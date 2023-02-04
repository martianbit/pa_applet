use std::rc::Rc;
use std::process::Command;
use cpp_core::Ptr;
use cpp_core::StaticUpcast;
use gtk::prelude::*;
use gtk::Dialog;
use gtk::Scale;
use gtk::PositionType;
use gtk::Orientation;
use gtk::Inhibit;
use gtk::StyleContext;
use gtk::CssProvider;
use gdk::Display;
use qt_core::QString;
use qt_core::QObject;
use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::slot;
use qt_gui::QIcon;
use qt_widgets::QApplication;
use qt_widgets::QSystemTrayIcon;

const MAX_ABS_VOLUME: u32 = 65536;
const DIALOG_WIDTH: u32 = 300;
const DIALOG_POS_X: u32 = 1920 - DIALOG_WIDTH - 10;
const DIALOG_POS_Y: u32 = 40;
const DIALOG_BORDER_SIZE: u32 = 1;
const DIALOG_BORDER_COLOR: &str = "#1B1B1B";
const DIALOG_BG_COLOR: &str = "#2F2F2F";
const FONT_NAME: &str = "Monaco";
const ICONS_NAME: &str = "Papirus-Dark";

static mut TRAY_ICON: Option<Rc<TrayIcon>> = None;
static mut CURRENT_ICON_NAME: Option<&str> = None;

#[derive(Copy, Clone)]
enum AudioFlow {
    Sink,
    Source,
}

impl AudioFlow {
    fn get_special_name_of_default(self) -> &'static str {
        match self {
            Self::Sink => "@DEFAULT_SINK@",
            Self::Source => "@DEFAULT_SOURCE@",
        }
    }
}

fn set_volume(volume: u8, audio_flow: AudioFlow) {
    let abs_volume = (volume as u32) * MAX_ABS_VOLUME / 100;

    Command::new("pactl")
        .arg(match audio_flow {
            AudioFlow::Sink => "set-sink-volume",
            AudioFlow::Source => "set-source-volume",
        })
        .arg(audio_flow.get_special_name_of_default())
        .arg(&abs_volume.to_string())
        .spawn().unwrap();
}

fn get_current_volume(audio_flow: AudioFlow) -> u8 {
    let abs_volume_raw = Command::new("pactl")
        .arg(match audio_flow {
            AudioFlow::Sink => "get-sink-volume",
            AudioFlow::Source => "get-source-volume",
        })
        .arg(audio_flow.get_special_name_of_default())
        .output().unwrap();

    let abs_volume: u32 = String::from_utf8(abs_volume_raw.stdout).unwrap()
        .split_whitespace()
        .nth(2).unwrap()
        .parse().unwrap();

    ((abs_volume as f64) * 100_f64 / (MAX_ABS_VOLUME as f64)).ceil() as u8
}

fn get_correct_icon_name(volume: u8) -> &'static str {
    if volume == 0 { "audio-volume-muted" }
    else if volume < 50 { "audio-volume-low" }
    else if volume < 100 { "audio-volume-medium" }
    else { "audio-volume-high" }
}

fn build_slider(audio_flow: AudioFlow) -> Scale {
    let slider = Scale::with_range(Orientation::Horizontal, 0_f64, 100_f64, 1_f64);

    slider.set_value(get_current_volume(audio_flow) as f64);
    slider.set_value_pos(PositionType::Left);

    slider.connect_value_changed(match audio_flow {
        AudioFlow::Sink => |x: &Scale| unsafe {
            let new_volume = x.value() as u8;

            set_volume(new_volume, AudioFlow::Sink);
            TRAY_ICON.as_ref().unwrap().update_icon(new_volume);
        },
        AudioFlow::Source => |x: &Scale| set_volume(x.value() as u8, AudioFlow::Source),
    });

    slider.connect_format_value(match audio_flow {
        AudioFlow::Sink => |_: &Scale, x| format!("sink {}", x),
        AudioFlow::Source => |_: &Scale, x| format!("srce {}", x),
    });

    slider
}

struct TrayIcon {
    widget: QBox<QSystemTrayIcon>,
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
        self.update_icon(get_current_volume(AudioFlow::Sink));

        self.widget.show();
    }

    unsafe fn update_icon(self: &Rc<Self>, volume: u8) {
        let icon_name = get_correct_icon_name(volume);

        if CURRENT_ICON_NAME == None || icon_name != CURRENT_ICON_NAME.unwrap() {
            self.widget.set_icon(&QIcon::from_theme_1a(&QString::from_std_str(icon_name)));
            CURRENT_ICON_NAME = Some(icon_name);
        }
    }

    #[slot(SlotNoArgs)]
    fn on_click(self: &Rc<Self>) {
        let sink_slider = build_slider(AudioFlow::Sink);
        let source_slider = build_slider(AudioFlow::Source);

        let dialog = Dialog::new();

        let content_area = dialog.content_area();

        dialog.move_(DIALOG_POS_X as i32, DIALOG_POS_Y as i32);
        dialog.resize(DIALOG_WIDTH as i32, 1);

        content_area.pack_start(&sink_slider, true, true, 0);
        content_area.pack_start(&source_slider, true, true, 0);

        dialog.connect_focus_out_event(|x, _| {
            x.emit_close();
            Inhibit(true)
        });

        dialog.show_all();
    }
}

fn main() {
    gtk::init().unwrap();

    let style_prov = CssProvider::new();

    style_prov.load_from_data(
        format!("*{{font-family:{};}}dialog{{border:{}px solid {};background-color:{};}}",
                FONT_NAME,
                DIALOG_BORDER_SIZE,
                DIALOG_BORDER_COLOR,
                DIALOG_BG_COLOR).as_bytes()
    ).unwrap();

    StyleContext::add_provider_for_screen(
        &Display::default().unwrap().default_screen(),
        &style_prov,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
    );

    QApplication::init(|_| unsafe {
        QIcon::set_theme_name(&QString::from_std_str(ICONS_NAME));
        TRAY_ICON = Some(TrayIcon::new());
        QApplication::exec()
    });
}

