use cosmic::{
    iced::Limits,
    iced::{self, wayland::InitialSurface, Application},
    iced_runtime::core::window,
    iced_style::application,
    theme::Theme,
};
use cosmic_applet::CosmicAppletHelper;
use freedesktop_desktop_entry::DesktopEntry;
use std::{env, fs, process::Command};

#[derive(Clone, Default)]
struct Desktop {
    name: String,
    icon: Option<String>,
    exec: String,
}

struct Button {
    desktop: Desktop,
    applet_helper: CosmicAppletHelper,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Msg {
    Press,
    Theme(Theme),
}

impl iced::Application for Button {
    type Message = Msg;
    type Theme = cosmic::Theme;
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = Desktop;

    fn new(desktop: Desktop) -> (Self, iced::Command<Msg>) {
        let applet_helper = CosmicAppletHelper::default();
        let theme = applet_helper.theme();
        (
            Button {
                desktop,
                applet_helper,
                theme,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Button")
    }

    fn close_requested(&self, _id: window::Id) -> Msg {
        unimplemented!()
    }

    fn style(&self) -> <Self::Theme as application::StyleSheet>::Style {
        <Self::Theme as application::StyleSheet>::Style::Custom(Box::new(|theme| {
            application::Appearance {
                background_color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0),
                text_color: theme.cosmic().on_bg_color().into(),
            }
        }))
    }

    fn subscription(&self) -> iced::Subscription<Msg> {
        self.applet_helper.theme_subscription(0).map(Msg::Theme)
    }

    fn update(&mut self, message: Msg) -> iced::Command<Msg> {
        match message {
            Msg::Press => {
                let _ = Command::new("sh").arg("-c").arg(&self.desktop.exec).spawn();
            }
            Msg::Theme(t) => {
                self.theme = t;
            }
        }
        iced::Command::none()
    }

    fn view(&self, _id: window::Id) -> cosmic::Element<Msg> {
        // TODO icon?
        cosmic::widget::button(cosmic::theme::Button::Text)
            .text(&self.desktop.name)
            .on_press(Msg::Press)
            .into()
    }
}

pub fn main() -> iced::Result {
    let id = env::args()
        .skip(1)
        .next()
        .expect("Requires desktop file id as argument.");

    let filename = format!("{id}.desktop");
    let mut desktop = None;
    for mut path in freedesktop_desktop_entry::default_paths() {
        path.push(&filename);
        if let Ok(bytes) = fs::read_to_string(&path) {
            if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
                desktop = Some(Desktop {
                    name: entry
                        .name(None)
                        .map(|x| x.to_string())
                        .expect(&format!("Desktop file '{filename}' doesn't have `Name`")),
                    icon: entry.icon().map(|x| x.to_string()),
                    exec: entry
                        .exec()
                        .map(|x| x.to_string())
                        .expect(&format!("Desktop file '{filename}' doesn't have `Exec`")),
                });
                break;
            }
        }
    }
    let desktop = desktop.expect(&format!(
        "Failed to find valid desktop file '{filename}' in search paths"
    ));
    let helper = CosmicAppletHelper::default();
    let mut settings = iced::Settings {
        flags: desktop,
        ..helper.window_settings()
    };
    match &mut settings.initial_surface {
        InitialSurface::XdgWindow(s) => {
            s.autosize = true;
            s.size_limits = Limits::NONE.min_height(1.0).min_width(1.0);
        }
        _ => unreachable!(),
    };
    Button::run(settings)
}