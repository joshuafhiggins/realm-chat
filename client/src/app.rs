use tarpc::context;
use tarpc::tokio_serde::formats::Json;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::log::*;
use realm_auth::types::RealmAuthClient;
use realm_shared::types::ErrorCode::*;
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
	
	#[serde(skip)]
	pub fetching_user_data_channel: (Sender<Result<ClientUser, ErrorCode>>, Receiver<Result<ClientUser, ErrorCode>>),
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
			
			fetching_user_data_channel: broadcast::channel(10),
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
					self.login_window_open = false;
					self.signup_window_open = false;
					self.login_window_code.clear();

					info!("Fetching user data...");
					let send_channel = self.fetching_user_data_channel.0.clone();
					let server_address = self.login_window_server_address.clone();
					let username = self.login_window_username.clone();
					
					let _handle = tokio::spawn(async move {
						let mut transport = tarpc::serde_transport::tcp::connect(&server_address, Json::default);
						transport.config_mut().max_frame_length(usize::MAX);

						let result = transport.await;
						let connection = match result {
							Ok(connection) => connection,
							Err(e) => {
								tracing::error!("Failed to connect to server: {}", e);
								return;
							}
						};

						let client = RealmAuthClient::new(tarpc::client::Config::default(), connection).spawn();
						let result = client.get_all_data(context::current(), username, token.clone()).await;

						match result {
							Ok(r) => {
								if let Err(code) = r {
									send_channel.send(Err(code)).unwrap();
								} else {
									let auth_user = r.unwrap();
									let servers: Vec<String> = {
										if auth_user.servers.eq("") {
											Vec::new()
										} else {
											auth_user.servers.split('|').map(|s| s.to_string()).collect()
										}
									};
									send_channel.send(Ok(ClientUser {
										id: auth_user.id,
										server_address,
										username: auth_user.username,
										email: auth_user.email,
										//avatar: auth_user.avatar,
										servers,
										token,
									})).unwrap();
								}
							},
							Err(_) => {
								send_channel.send(Err(RPCError)).unwrap();
							},
						};
					});
				},
				Err(e) => tracing::error!("Error in login flow: {:?}", e),
			}
		}

		while let Ok(result) = self.fetching_user_data_channel.1.try_recv() {
			match result {
				Ok(client_user) => {
					info!("Got data! User: {:?}", client_user);
					self.current_user = Some(client_user);
				},
				Err(e) => error!("Error in login flow: {:?}", e),
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