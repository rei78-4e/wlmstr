use gtk::prelude::*;
use gtk::{gio, glib};
use std::path::Path;
use std::rc::Rc;
use wlmstr::Derection;
use wlmstr::status::{Status, is_image};

const APP_ID: &str = "io.github.Uliboooo.wlmstr";

struct Ui {
    window: gtk::ApplicationWindow,
    preview: gtk::Stack,
    picture: gtk::Picture,
    video: gtk::Video,
    name_label: gtk::Label,
    info_label: gtk::Label,
    buttons: Vec<gtk::Button>,
    awww_args: Vec<String>,
}

impl Ui {
    fn show_status(&self, st: &Status) {
        let path = st.get_current_p_path();
        self.name_label.set_label(&file_name(&path));
        self.info_label
            .set_label(&st.get_dir_path().to_string_lossy());
        self.info_label.remove_css_class("error");
        self.set_preview(&path);
    }

    fn show_error(&self, msg: &str) {
        self.info_label.set_label(msg);
        self.info_label.add_css_class("error");
    }

    fn set_preview(&self, path: &Path) {
        // stop a previously playing video before switching
        self.video.set_media_stream(None::<&gtk::MediaStream>);
        match is_image(path) {
            Ok(true) => {
                self.picture.set_filename(Some(path));
                self.preview.set_visible_child_name("image");
            }
            Ok(false) => {
                let media = gtk::MediaFile::for_filename(path);
                media.set_muted(true);
                media.set_loop(true);
                media.play();
                self.video.set_media_stream(Some(&media));
                self.preview.set_visible_child_name("video");
            }
            Err(_) => self.preview.set_visible_child_name("empty"),
        }
    }

    fn set_busy(&self, busy: bool) {
        for b in &self.buttons {
            b.set_sensitive(!busy);
        }
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn on_step(ui: &Rc<Ui>, derection: Derection) {
    ui.set_busy(true);
    let args = ui.awww_args.clone();
    let ui = ui.clone();
    glib::spawn_future_local(async move {
        // awww / mpvpaper are blocking processes, so run them off the main thread
        let res =
            gio::spawn_blocking(move || wlmstr::step(derection, args).map_err(|e| e.to_string()))
                .await;
        match res {
            Ok(Ok(st)) => ui.show_status(&st),
            Ok(Err(e)) => ui.show_error(&e),
            Err(_) => ui.show_error("worker thread panicked"),
        }
        ui.set_busy(false);
    });
}

fn on_choose_dir(ui: &Rc<Ui>) {
    let ui = ui.clone();
    glib::spawn_future_local(async move {
        let dialog = gtk::FileDialog::builder()
            .title("Select wallpaper directory")
            .modal(true)
            .build();
        if let Ok(st) = wlmstr::load_status() {
            dialog.set_initial_folder(Some(&gio::File::for_path(st.get_dir_path())));
        }
        let Ok(folder) = dialog.select_folder_future(Some(&ui.window)).await else {
            return; // cancelled
        };
        let Some(dir) = folder.path() else {
            return ui.show_error("selected folder is not a local path");
        };
        match Status::get_first_paper_in_dir(&dir)
            .and_then(|first| wlmstr::set(Some(dir), Some(first)))
        {
            Ok(st) => {
                ui.show_status(&st);
                ui.set_busy(false);
            }
            Err(e) => ui.show_error(&e.to_string()),
        }
    });
}

fn build_ui(app: &gtk::Application) {
    let awww_args = wlmstr::resolve_config().unwrap_or_default().awww.args;

    let picture = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::Contain)
        .build();
    let video = gtk::Video::builder().autoplay(true).build();
    let empty = gtk::Label::new(Some("No preview"));
    empty.add_css_class("dim-label");

    let preview = gtk::Stack::builder()
        .vexpand(true)
        .hexpand(true)
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    preview.add_named(&picture, Some("image"));
    preview.add_named(&video, Some("video"));
    preview.add_named(&empty, Some("empty"));
    preview.set_visible_child_name("empty");

    let name_label = gtk::Label::builder()
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .build();
    name_label.add_css_class("title-3");
    let info_label = gtk::Label::builder()
        .wrap(true)
        .justify(gtk::Justification::Center)
        .build();
    info_label.add_css_class("dim-label");

    let prev_btn = gtk::Button::from_icon_name("go-previous-symbolic");
    prev_btn.set_tooltip_text(Some("Previous"));
    let rnd_btn = gtk::Button::from_icon_name("media-playlist-shuffle-symbolic");
    rnd_btn.set_tooltip_text(Some("Random"));
    let next_btn = gtk::Button::from_icon_name("go-next-symbolic");
    next_btn.set_tooltip_text(Some("Next"));

    let controls = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .build();
    controls.append(&prev_btn);
    controls.append(&rnd_btn);
    controls.append(&next_btn);

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&preview);
    content.append(&name_label);
    content.append(&controls);
    content.append(&info_label);

    let dir_btn = gtk::Button::from_icon_name("folder-open-symbolic");
    dir_btn.set_tooltip_text(Some("Select wallpaper directory"));
    let header = gtk::HeaderBar::new();
    header.pack_start(&dir_btn);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title(wlmstr::PROGRAM_NAME)
        .default_width(720)
        .default_height(520)
        .child(&content)
        .build();
    window.set_titlebar(Some(&header));

    let ui = Rc::new(Ui {
        window,
        preview,
        picture,
        video,
        name_label,
        info_label,
        buttons: vec![prev_btn.clone(), rnd_btn.clone(), next_btn.clone()],
        awww_args,
    });

    for (btn, derection) in [
        (&prev_btn, Derection::Previous),
        (&rnd_btn, Derection::Random),
        (&next_btn, Derection::Sequence),
    ] {
        let ui = ui.clone();
        btn.connect_clicked(move |_| on_step(&ui, derection.clone()));
    }
    {
        let ui = ui.clone();
        dir_btn.connect_clicked(move |_| on_choose_dir(&ui));
    }

    match wlmstr::load_status() {
        Ok(st) => ui.show_status(&st),
        Err(_) => {
            ui.set_busy(true);
            ui.show_error("Wallpaper directory is not set. Select one with the folder button.");
        }
    }

    ui.window.present();
}

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
