/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RealmApp {
    // Example stuff:
    label: String,
    selected: bool,

    // #[serde(skip)] // This how you opt-out of serialization of a field
    // value: f32,
}

impl Default for RealmApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            selected: false
        }
    }
}

impl RealmApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for RealmApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     egui::menu::bar(ui, |ui| {
        //         egui::widgets::global_dark_light_mode_buttons(ui);
        //     });
        // });

        //Servers
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label("test");
            let response = ui.selectable_label(self.selected, "bruh");
            if response.clicked() {
                self.selected = !self.selected;
            }
        });

        //Channels
        egui::SidePanel::left("inner_left_panel").show(ctx, |ui| {
            ui.label("inner");
        });

        //Conversation
        egui::CentralPanel::default().show(ctx, |ui| {
            //TODO: Messages

            //Message Box
            ui.with_layout(egui::Layout::bottom_up(egui::Align::BOTTOM).with_cross_justify(true), |ui| {
                let response = ui.add(egui::TextEdit::multiline(&mut self.label).desired_rows(1));
                if response.changed() {

                }
                ui.separator();

            });

            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     egui::warn_if_debug_build(ui);
            // });
        });
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}