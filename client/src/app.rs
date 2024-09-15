use crate::ui::panels;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
	// Example stuff:
	pub label: String,
	pub selected: bool,
	pub selected_serverid: Option<String>,
	pub selected_roomid: Option<String>,

	#[serde(skip)] // This how you opt-out of serialization of a field
	pub value: f32,
}

impl Default for TemplateApp {
	fn default() -> Self {
		Self {
			// Example stuff:
			label: "Hello World!".to_owned(),
			selected: false,
			selected_serverid: None,
			selected_roomid: None,
			value: 2.7,
		}
	}
}

impl TemplateApp {
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

impl eframe::App for TemplateApp {
	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		// File -> Quit
		panels::top_panel(ctx);
		
		panels::servers(self, ctx);
		
		panels::rooms(self, ctx);

		panels::messages(self, ctx)
	}

	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}
}