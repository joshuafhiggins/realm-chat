use tarpc::context;
use tarpc::tokio_serde::formats::Json;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::log::*;
use realm_auth::types::RealmAuthClient;
use realm_server::types::{RealmChatClient, User};
use realm_shared::stoken;
use realm_shared::types::ErrorCode::*;
use realm_shared::types::ErrorCode;
use crate::types::{CServer, CUser};
use crate::ui::gui;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RealmApp {
	// Example stuff:
	pub label: String,
	pub selected: bool,
	#[serde(skip)]
	pub selected_serverid: String,
	#[serde(skip)]
	pub selected_roomid: String,

	#[serde(skip)]
	pub current_user: Option<CUser>,

	pub saved_username: Option<String>,
	pub saved_token: Option<String>,
	pub saved_auth_address: Option<String>,
	
	#[serde(skip)]
	pub active_servers: Option<Vec<CServer>>,

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
	pub server_window_open: bool,
	#[serde(skip)]
	pub server_window_domain: String,
	#[serde(skip)]
	pub server_window_port: String,

	#[serde(skip)]
	pub login_start_channel: (Sender<Result<(), ErrorCode>>, Receiver<Result<(), ErrorCode>>),
	#[serde(skip)]
	pub login_ending_channel: (Sender<Result<String, ErrorCode>>, Receiver<Result<String, ErrorCode>>),

	#[serde(skip)]
	pub fetching_user_data_channel: (Sender<Result<CUser, ErrorCode>>, Receiver<Result<CUser, ErrorCode>>),

	#[serde(skip)]
	pub add_server_channel: (Sender<Result<String, ErrorCode>>, Receiver<Result<String, ErrorCode>>),
	#[serde(skip)]
	pub join_server_channel: (Sender<Result<(), ErrorCode>>, Receiver<Result<(), ErrorCode>>),

	#[serde(skip)]
	pub fetching_servers_channel: (Sender<Result<CServer, ErrorCode>>, Receiver<Result<CServer, ErrorCode>>),
}

impl Default for RealmApp {
	fn default() -> Self {
		Self {
			// Example stuff:
			label: "Hello World!".to_owned(),
			selected: false,
			selected_serverid: String::new(),
			selected_roomid: String::new(),
			current_user: None,
			saved_username: None,
			saved_token: None,
			saved_auth_address: None,
			active_servers: None,
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

			server_window_open: false,
			server_window_domain: String::new(),
			server_window_port: "5051".to_string(),

			fetching_user_data_channel: broadcast::channel(10),
			add_server_channel: broadcast::channel(10),
			join_server_channel: broadcast::channel(10),
			fetching_servers_channel: broadcast::channel(10),
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

pub fn fetch_user_data(send_channel: Sender<Result<CUser, ErrorCode>>, server_address: String, username: String, token: String) {
	let _handle = tokio::spawn(async move {
		let mut transport = tarpc::serde_transport::tcp::connect(&server_address, Json::default);
		transport.config_mut().max_frame_length(usize::MAX);

		let result = transport.await;
		let auth_connection = match result {
			Ok(connection) => connection,
			Err(_) => {
				send_channel.send(Err(UnableToConnectToServer)).unwrap();
				return;
			}
		};

		let client = RealmAuthClient::new(tarpc::client::Config::default(), auth_connection).spawn();
		let result = client.get_all_data(context::current(), username, token.clone()).await;

		match result {
			Ok(r) => {
				if let Err(code) = r {
					send_channel.send(Err(code)).unwrap();
				} else {
					let auth_user = r.unwrap();
					send_channel.send(Ok(CUser {
						id: auth_user.id,
						auth_address: server_address,
						username: auth_user.username,
						email: auth_user.email,
						//avatar: auth_user.avatar,
						server_addresses: auth_user.servers.split('|').map(|s| s.to_string()).collect(),
						token,
					})).unwrap();
				}
			}
			Err(_) => {
				send_channel.send(Err(RPCError)).unwrap();
			}
		};
	});
}

pub fn fetch_server_data(addresses: Vec<String>, channel: Sender<Result<CServer, ErrorCode>>) {
	for server_address in addresses {
		let send_channel = channel.clone();

		let _handle = tokio::spawn(async move {
			let mut transport = tarpc::serde_transport::tcp::connect(&server_address, Json::default);
			transport.config_mut().max_frame_length(usize::MAX);

			let result = transport.await;
			let connection = match result {
				Ok(connection) => connection,
				Err(_) => {
					send_channel.send(Err(UnableToConnectToServer)).unwrap();
					return;
				}
			};

			let client = RealmChatClient::new(tarpc::client::Config::default(), connection).spawn();
			let info = client.get_info(context::current()).await.unwrap();
			let is_admin = client.is_user_admin(context::current(), info.server_id.clone()).await.unwrap();
			let is_owner = client.is_user_owner(context::current(), info.server_id.clone()).await.unwrap();
			send_channel.send(Ok(CServer {
				server_id: info.server_id,
				domain: server_address.split(':').collect::<Vec<&str>>()[0].to_string(),
				port: server_address.split(':').collect::<Vec<&str>>()[1].to_string().parse::<u16>().unwrap(),
				is_admin,
				is_owner,
			})).unwrap();
		});
	}
}

impl eframe::App for RealmApp {
	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		// Init at launch and refresh our user data from the auth server
		if self.current_user.is_none() && 
			self.saved_token.is_some() && 
			self.saved_auth_address.is_some() &&
			self.saved_username.is_some()
		{
			let send_channel = self.fetching_user_data_channel.0.clone();
			let server_address = self.saved_auth_address.clone().unwrap();
			let username = self.saved_username.clone().unwrap();
			let token = self.saved_token.clone().unwrap();
			fetch_user_data(send_channel, server_address, username, token);
		}

		// Fetching servers
		if self.active_servers.is_none() && self.current_user.is_some() {
			self.active_servers = Some(Vec::new());
			fetch_server_data(self.current_user.clone().unwrap().server_addresses.clone(), self.fetching_servers_channel.0.clone());
		}

		// Starting the login flow
		while let Ok(result) = self.login_start_channel.1.try_recv() {
			match result {
				Ok(_) => self.login_ready_for_code_input = true,
				Err(e) => tracing::error!("Error in login/account creation flow: {:?}", e),
			}
		}

		// End of the login flow
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

					self.saved_token = Some(token.clone());
					self.saved_username = Some(username.clone());
					self.saved_auth_address = Some(server_address.clone());
					
					fetch_user_data(send_channel, server_address, username, token);
				}
				Err(e) => tracing::error!("Error in login flow: {:?}", e),
			}
		}

		// Fetching user data
		while let Ok(result) = self.fetching_user_data_channel.1.try_recv() {
			match result {
				Ok(client_user) => {
					info!("Got data! User: {:?}", client_user);
					self.current_user = Some(client_user);
				}
				Err(e) => error!("Error fetching data: {:?}", e),
			}
		}

		// Adding a server
		while let Ok(result) = self.add_server_channel.1.try_recv() {
			match result {
				Ok(address) => {
					info!("New server added at: {:?}", address);
					self.server_window_open = false;

					let send_channel = self.join_server_channel.0.clone();
					let auth_address = self.saved_auth_address.clone().unwrap();
					let username = self.saved_username.clone().unwrap();
					let token = self.saved_token.clone().unwrap();
					
					let thread_username = username.clone();
					let thread_token = token.clone();
					let _handle = tokio::spawn(async move {
						let mut transport = tarpc::serde_transport::tcp::connect(&address, Json::default);
						transport.config_mut().max_frame_length(usize::MAX);

						let result = transport.await;
						let connection = match result {
							Ok(connection) => connection,
							Err(_) => {
								send_channel.clone().send(Err(UnableToConnectToServer)).unwrap();
								return;
							}
						};

						let client = RealmChatClient::new(tarpc::client::Config::default(), connection).spawn();
						
						let domain = address.split(':').collect::<Vec<&str>>()[0].to_string();
						let port = address.split(':').collect::<Vec<&str>>()[1].to_string().parse::<u16>().unwrap();
						
						let info = client.get_info(context::current()).await.unwrap();
						
						let result = client.join_server(context::current(), stoken(&thread_token, &info.server_id, &domain, port), thread_username).await;
						
						match result {
							Ok(_) => {
								info!("Joined server!");
							},
							Err(e) => error!("Error joining server: {:?}", e),
						}
					});

					fetch_user_data(self.fetching_user_data_channel.0.clone(), auth_address, username, token);
				}
				Err(e) => error!("Error in adding a server: {:?}", e),
			}
		}
		
		// Joining a server
		while let Ok(result) = self.join_server_channel.1.try_recv() {
			match result {
				Ok(_) => {
					info!("Successfully joined a server");
					fetch_server_data(self.current_user.clone().unwrap().server_addresses.clone(), self.fetching_servers_channel.0.clone());
				},
				Err(code) => {
					error!("Error joining server: {:?}", code);
				}
			}
		}

		// Fetching servers
		while let Ok(result) = self.fetching_servers_channel.1.try_recv() {
			match result {
				Ok(server) => {
					info!("Got server data! Server: {:?}", server);
					if let Some(active_servers) = &mut self.active_servers {
						active_servers.push(server);
					}
				}
				Err(e) => error!("Error fetching server data: {:?}", e),
			}
		}

		// File -> Quit
		gui::top_panel(self, ctx);

		gui::servers(self, ctx);

		gui::rooms(self, ctx);

		gui::messages(self, ctx);

		gui::modals(self, ctx)
	}

	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}
}