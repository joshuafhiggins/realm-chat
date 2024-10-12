use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::log::info;
use realm_shared::types::ErrorCode;
use crate::types::ClientUser;
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
	
	pub current_user: Option<ClientUser>,

	#[serde(skip)]
	pub value: f32,
	#[serde(skip)]
	pub login_window_open: bool,
	#[serde(skip)]
	pub login_window_username: String,
	#[serde(skip)]
	pub login_window_code: String,
	#[serde(skip)]
	pub login_window_server_address: String,
	#[serde(skip)]
	pub login_window_email: String,
	
	#[serde(skip)]
	pub login_ready_for_code_input: bool,

	#[serde(skip)]
	pub signup_window_open: bool,
	
	#[serde(skip)]
	pub login_start_channel: (Sender<Result<(), ErrorCode>>, Receiver<Result<(), ErrorCode>>),
	#[serde(skip)]
	pub login_ending_channel: (Sender<Result<String, ErrorCode>>, Receiver<Result<String, ErrorCode>>),
}

impl Default for TemplateApp {
	fn default() -> Self {
		Self {
			// Example stuff:
			label: "Hello World!".to_owned(),
			selected: false,
			selected_serverid: None,
			selected_roomid: None,
			current_user: None,
			value: 2.7,
			
			login_window_open: false,
			login_window_username: String::new(),
			login_window_code: String::new(),
			login_window_server_address: String::new(),
			login_start_channel: broadcast::channel(10),
			login_ending_channel: broadcast::channel(10),
			login_ready_for_code_input: false,
			login_window_email: String::new(),
			
			signup_window_open: false,
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

		while let Ok(result) = self.login_start_channel.1.try_recv() {
			match result {
				Ok(_) => self.login_ready_for_code_input = true,
				Err(e) => tracing::error!("Error in login/account creation flow: {:?}", e),
			}
		}

		while let Ok(result) = self.login_ending_channel.1.try_recv() {
			match result {
				Ok(token) => {
					info!("Login successful! Token: {token}");
					self.login_ready_for_code_input = false;
					
				},
				Err(e) => tracing::error!("Error in login flow: {:?}", e),
			}
		}

		// File -> Quit
		panels::top_panel(self, ctx);
		
		panels::servers(self, ctx);
		
		panels::rooms(self, ctx);

		panels::messages(self, ctx)
	}

	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}
}