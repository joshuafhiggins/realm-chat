use std::time::Duration;
use tarpc::context;
use tarpc::tokio_serde::formats::Json;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::log::*;
use realm_auth::types::RealmAuthClient;
use realm_server::events::Event;
use realm_server::types::{RealmChatClient, Room};
use realm_shared::stoken;
use realm_shared::types::ErrorCode::*;
use realm_shared::types::ErrorCode;
use crate::types::{CServer, CUser};
use crate::ui::gui;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RealmApp {
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
	pub text_message_input: String,
	#[serde(skip)]
	pub login_window_open: bool,
	#[serde(skip)]
	pub login_window_username: String,
	#[serde(skip)]
	pub login_window_code: String,
	#[serde(skip)]
	pub login_window_server_domain: String,
	#[serde(skip)]
	pub login_window_server_port: String,
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
	pub room_window_open: bool,
	#[serde(skip)]
	pub room_window_name: String,
	#[serde(skip)]
	pub room_window_admin_only_send: bool,
	#[serde(skip)]
	pub room_window_admin_only_view: bool,
	
	#[serde(skip)]
	pub info_window_open: bool,

	#[serde(skip)]
	pub login_start_channel: (Sender<Result<(), ErrorCode>>, Receiver<Result<(), ErrorCode>>),
	#[serde(skip)]
	pub login_ending_channel: (Sender<Result<String, ErrorCode>>, Receiver<Result<String, ErrorCode>>),

	#[serde(skip)]
	pub fetching_user_data_channel: (Sender<Result<CUser, ErrorCode>>, Receiver<Result<CUser, ErrorCode>>),

	#[serde(skip)]
	pub add_server_channel: (Sender<Result<String, ErrorCode>>, Receiver<Result<String, ErrorCode>>),
	#[serde(skip)]
	pub remove_server_channel: (Sender<Result<(), ErrorCode>>, Receiver<Result<(), ErrorCode>>),
	#[serde(skip)]
	pub join_server_channel: (Sender<Result<(), ErrorCode>>, Receiver<Result<(), ErrorCode>>),
	#[serde(skip)]
	pub leave_server_channel: (Sender<Result<(String, String, u16), ErrorCode>>, Receiver<Result<(String, String, u16), ErrorCode>>),

	#[serde(skip)]
	pub fetching_servers_channel: (Sender<Result<CServer, ErrorCode>>, Receiver<Result<CServer, ErrorCode>>),
	
	#[serde(skip)]
	pub add_room_channel: (Sender<Result<CServer, ErrorCode>>, Receiver<Result<CServer, ErrorCode>>),
	#[serde(skip)]
	pub delete_room_channel: (Sender<Result<CServer, ErrorCode>>, Receiver<Result<CServer, ErrorCode>>),
	#[serde(skip)]
	pub room_changes_channel: (Sender<Result<(CServer, Vec<Room>), ErrorCode>>, Receiver<Result<(CServer, Vec<Room>), ErrorCode>>),

	#[serde(skip)]
	pub event_channel: (Sender<(String, (u32, Event))>, Receiver<(String, (u32, Event))>),
	#[serde(skip)]
	pub polling_threads: Vec<(String, JoinHandle<()>)>,
}

impl Default for RealmApp {
	fn default() -> Self {
		Self {
			selected_serverid: String::new(),
			selected_roomid: String::new(),
			current_user: None,
			saved_username: None,
			saved_token: None,
			saved_auth_address: None,
			active_servers: None,
			text_message_input: String::new(),

			login_window_open: false,
			login_window_username: String::new(),
			login_window_code: String::new(),
			login_window_server_domain: "auth.realm.abunchofknowitalls.com".to_string(),
			login_window_server_port: "5052".to_string(),
			login_start_channel: broadcast::channel(256),
			login_ending_channel: broadcast::channel(256),
			login_ready_for_code_input: false,
			login_window_email: String::new(),

			signup_window_open: false,

			server_window_open: false,
			server_window_domain: "realm.abunchofknowitalls.com".to_string(),
			server_window_port: "5051".to_string(),
			
			room_window_open: false,
			room_window_name: String::new(),
			room_window_admin_only_send: false,
			room_window_admin_only_view: false,
			
			info_window_open: false,

			fetching_user_data_channel: broadcast::channel(256),
			add_server_channel: broadcast::channel(256),
			remove_server_channel: broadcast::channel(256),
			join_server_channel: broadcast::channel(256),
			leave_server_channel: broadcast::channel(256),
			fetching_servers_channel: broadcast::channel(256),
			add_room_channel: broadcast::channel(256),
			delete_room_channel: broadcast::channel(256),
			room_changes_channel: broadcast::channel(256),
			event_channel: broadcast::channel(256),
			polling_threads: Vec::new(),
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
		if !cfg!(debug_assertions) {
			if let Some(storage) = cc.storage {
				return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
			}
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

pub fn fetch_server_data(channel: Sender<Result<CServer, ErrorCode>>, addresses: Vec<String>, token: String, username: String){
	for server_address in addresses {
		let send_channel = channel.clone();
		let token = token.clone();
		let userid = username.clone();

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
			let domain = server_address.split(':').collect::<Vec<&str>>()[0].to_string();
			let port = server_address.split(':').collect::<Vec<&str>>()[1].to_string().parse::<u16>().unwrap();
			let stoken = stoken(&token, &info.server_id, &domain, port);
			let is_admin = client.is_user_admin(context::current(), userid.clone()).await.unwrap();
			let is_owner = client.is_user_owner(context::current(), userid.clone()).await.unwrap();
			let rooms = client.get_rooms(context::current(), stoken.clone(), userid.clone()).await.unwrap().unwrap();
			send_channel.send(Ok(CServer {
				tarpc_conn: client,
				server_id: info.server_id,
				domain,
				port,
				is_admin,
				is_owner,
				last_event_index: 0,
				messages: Vec::new(),
				rooms,
			})).unwrap();
		});
	}
}

pub fn fetch_rooms_data(send_channel: Sender<Result<(CServer, Vec<Room>), ErrorCode>>, server: CServer, token: String, userid: String) {
	let _handle = tokio::spawn(async move {
		let result = server.tarpc_conn.get_rooms(
			context::current(),
			stoken(&token, &server.server_id, &server.domain, server.port),
			userid
		).await;
		
		match result {
			Ok(r) => {
				if let Ok(rooms) = r {
					send_channel.send(Ok((server, rooms))).unwrap();
				} else { 
					send_channel.send(Err(r.unwrap_err())).unwrap();
				}
			}
			Err(_) => { send_channel.send(Err(RPCError)).unwrap(); }
		}
	});
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
			fetch_server_data(
				self.fetching_servers_channel.0.clone(),
				self.current_user.as_ref().unwrap().server_addresses.clone(), 
				self.current_user.as_ref().unwrap().token.clone(),
				self.current_user.as_ref().unwrap().username.clone()
			);
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
					let server_address = format!("{}:{}", self.login_window_server_domain, self.login_window_server_port);
					let port = self.login_window_server_port.clone().parse::<u16>().unwrap();
					let username = format!("@{}:{}", self.login_window_username, self.login_window_server_domain);

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
					self.current_user.replace(client_user);
				}
				Err(e) => error!("Error fetching data: {:?}", e),
			}
		}

		// Adding a server (auth)
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
							Ok(r) => {
								info!("Joined server!");
								match r {
									Ok(_) => { send_channel.send(Ok(())).unwrap(); },
									Err(e) => { send_channel.send(Err(e)).unwrap(); },
								}
							},
							Err(_) => {
								error!("Error joining server");
								send_channel.send(Err(RPCError)).unwrap();
							},
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
					fetch_server_data(
						self.fetching_servers_channel.0.clone(),
						self.current_user.as_ref().unwrap().server_addresses.clone(), 
						self.current_user.as_ref().unwrap().token.clone(),
						self.current_user.as_ref().unwrap().username.clone()
					);
				},
				Err(code) => {
					error!("Error joining server: {:?}", code);
				}
			}
		}
		
		// Leaving a server
		while let Ok(result) = self.leave_server_channel.1.try_recv() {
			match result {
				Ok((serverid, domain, port)) => {
					info!("Successfully left a server");
					self.active_servers.as_mut().unwrap().retain(|s| !s.server_id.eq(&serverid));
					self.selected_serverid.clear();
					self.selected_roomid.clear();
					let send_channel = self.remove_server_channel.0.clone();
					let auth_address = self.current_user.clone().unwrap().auth_address;
					let username = self.current_user.clone().unwrap().username;
					let token = self.current_user.clone().unwrap().token;
					let _handle = tokio::spawn(async move {
						let mut transport = tarpc::serde_transport::tcp::connect(&auth_address, Json::default);
						transport.config_mut().max_frame_length(usize::MAX);

						let result = transport.await;
						let connection = match result {
							Ok(connection) => connection,
							Err(_) => {
								send_channel.clone().send(Err(UnableToConnectToServer)).unwrap();
								return;
							}
						};

						let client = RealmAuthClient::new(tarpc::client::Config::default(), connection).spawn();
						
						let result = client.remove_server(context::current(), username, token, domain, port).await;
						match result {
							Ok(r) => { send_channel.send(r).unwrap(); },
							Err(_) => { send_channel.send(Err(RPCError)).unwrap(); },
						}
					});
				},
				Err(code) => {
					error!("Error leaving server: {:?}", code);
				}
			}
		}
		
		// Removing a server (auth)
		while let Ok(result) = self.remove_server_channel.1.try_recv() {
			match result {
				Ok(_) => {
					let send_channel = self.fetching_user_data_channel.0.clone();
					let server_address = self.saved_auth_address.clone().unwrap();
					let username = self.saved_username.clone().unwrap();
					let token = self.saved_token.clone().unwrap();
					fetch_user_data(send_channel, server_address, username, token);
					info!("Successfully removed a server");
				}
				Err(code) => {
					error!("Error removing server: {:?}", code);
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
		
		// Added Room
		while let Ok(result) = self.add_room_channel.1.try_recv() {
			match result {
				Ok(server) => {
					info!("Got room add! Fetching them...");
					fetch_rooms_data(
						self.room_changes_channel.0.clone(), 
						server, 
						self.current_user.as_ref().unwrap().token.clone(),
						self.current_user.as_ref().unwrap().username.clone()
					);
					self.room_window_open = false;
				}
				Err(e) => error!("Error adding room: {:?}", e),
			}
		}
		
		// Deleting a room
		while let Ok(result) = self.delete_room_channel.1.try_recv() {
			match result {
				Ok(server) => {
					info!("Got room delete! Fetching them...");
					self.selected_roomid.clear();
					fetch_rooms_data(
						self.room_changes_channel.0.clone(), 
						server, 
						self.current_user.as_ref().unwrap().token.clone(),
						self.current_user.as_ref().unwrap().username.clone()
					);
				}
				Err(e) => error!("Error deleting room: {:?}", e),
			}
		}
		
		// Fetching rooms
		while let Ok(result) = self.room_changes_channel.1.try_recv() {
			match result {
				Ok(tuple) => {
					info!("Got room data for a server: {:?}", tuple);
					if let Some(servers) = &mut self.active_servers {
						for server in servers {
							if server.server_id.eq(&tuple.0.server_id) {
								server.rooms = tuple.1.clone();
							}
						}
					}
				}
				Err(e) => error!("Error fetching room data: {:?}", e),
			}
		}
		
		// Polling events
		while let Ok((serverid, (index, event))) = self.event_channel.1.try_recv() {
			if let Some(active_servers) = &mut self.active_servers {
				for server in active_servers {
					if server.server_id.eq(&serverid) {
						match event.clone() {
							Event::NewMessage(message) => {
								server.messages.push(message);
							}
							Event::NewRoom(room) => {
								server.rooms.push(room);
							}
							Event::DeleteRoom(roomid) => {
								server.rooms.retain(|r| !r.roomid.eq(&roomid));
								if self.selected_roomid.eq(&roomid) {
									self.selected_roomid.clear();
								}
							}
						}
						server.last_event_index = index;
					}
				}
			}
		}
		
		// Manage polling threads
		if let Some(active_servers) = &mut self.active_servers {
			if self.polling_threads.len() != active_servers.len() {
				let running_thread_serverids = self.polling_threads.iter().map(|t| t.0.clone()).collect::<Vec<String>>();
				let missing_servers = active_servers.clone().into_iter().filter(|s| !running_thread_serverids.contains(&s.server_id)).collect::<Vec<CServer>>();
				for server in missing_servers {
					let send_channel = self.event_channel.0.clone();
					let _handle = tokio::spawn(async move {
						let mut transport = tarpc::serde_transport::tcp::connect(format!("{}:{}", server.domain, server.port), Json::default);
						transport.config_mut().max_frame_length(usize::MAX);

						let result = transport.await;
						let connection = match result {
							Ok(connection) => connection,
							Err(_) => {
								return;
							}
						};

						let client = RealmChatClient::new(tarpc::client::Config::default(), connection).spawn();
						loop {
							let result = client.poll_events_since(
								context::current(),
								server.last_event_index
							).await;
							
							match result {
								Ok(events) => {
									for event in events {
										send_channel.send((server.server_id.clone(), (event.0, event.1))).unwrap();
									}
								}
								Err(_) => break,
							}

							sleep(Duration::from_millis(1000)).await;
						}
					});
				}
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
		if !cfg!(debug_assertions) {
			eframe::set_value(storage, eframe::APP_KEY, self);
		}
	}
}