use egui::{Context, SelectableLabel};
use tarpc::context;
use tarpc::tokio_serde::formats::Json;
use realm_auth::types::RealmAuthClient;
use realm_shared::types::ErrorCode::RPCError;
use regex::Regex;
use tracing::log::*;
use realm_server::types::{RealmChatClient, Room};
use realm_shared::stoken;
use realm_shared::types::ErrorCode;
use crate::app::RealmApp;
use crate::types::CServer;

pub fn top_panel(app: &mut RealmApp, ctx: &Context) {
	egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
		egui::menu::bar(ui, |ui| {
			if app.current_user.is_none() && ui.button("Sign Up").clicked() {
				app.signup_window_open = true;
			}

			if app.current_user.is_none() && ui.button("Login").clicked() {
				app.login_window_open = true;
			}

			if app.current_user.is_some() && ui.button("Logout").clicked() {
				let address = app.current_user.clone().unwrap().auth_address;
				let username = app.current_user.clone().unwrap().username;
				let token = app.current_user.clone().unwrap().token;

				let _handle = tokio::spawn(async move {
					let mut transport = tarpc::serde_transport::tcp::connect(&address, Json::default);
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
					let result = client.sign_out(context::current(), username, token).await;

					match result {
						Ok(_) => info!("Signed out!"), // TODO: properly handle this
						Err(e) => error!("Error signing out: {:?}", e),
					}
				});

				app.current_user = None;
				app.saved_username = None;
				app.saved_token = None;
				app.saved_auth_address = None;
				
				app.active_servers = None;
				app.selected_roomid.clear();
				app.selected_serverid.clear();
			}

			if ui.button("Quit").clicked() {
				ctx.send_viewport_cmd(egui::ViewportCommand::Close);
			}

			ui.add_space(16.0);

			egui::widgets::global_theme_preference_buttons(ui);
		});
	});
}

pub fn servers(app: &mut RealmApp, ctx: &Context) {
	egui::SidePanel::left("servers").show(ctx, |ui| {
		ui.horizontal(|ui| {
			ui.heading("Servers");
			if app.current_user.is_some() && ui.button("+").clicked() {
				app.server_window_open = true;
			}
			if !app.selected_serverid.is_empty() && ui.button("-").clicked() {
				let server = app.active_servers.clone().unwrap().into_iter().find(|s| s.server_id.eq(&app.selected_serverid)).unwrap();
				let token = app.current_user.as_ref().unwrap().token.clone();
				let userid = app.current_user.as_ref().unwrap().username.clone();
				let send_channel = app.leave_server_channel.0.clone();
				let _handle = tokio::spawn(async move {
					let result = server.tarpc_conn.leave_server(
						context::current(),
						stoken(&token, &server.server_id, &server.domain, server.port),
						userid
					).await;

					match result {
						Ok(r) => {
							match r {
								Ok(_) => send_channel.send(Ok((server.server_id.clone(), server.domain.clone(), server.port))).unwrap(),
								Err(e) => send_channel.send(Err(e)).unwrap()
							}
						},
						Err(_) => send_channel.send(Err(RPCError)).unwrap(),
					}
				});
			}
		});
		ui.separator();

		if let Some(active_servers) = &mut app.active_servers {
			for server in active_servers {
				if ui.add(SelectableLabel::new(server.server_id.eq(&app.selected_serverid), server.server_id.clone())).clicked() {
					if app.selected_serverid.eq(&server.server_id) {
						app.selected_serverid.clear();
					} else {
						app.selected_serverid = server.server_id.clone();
					}
					app.selected_roomid.clear();
				}
			}
		}
	});
}

pub fn rooms(app: &mut RealmApp, ctx: &Context) {
	egui::SidePanel::left("rooms").show(ctx, |ui| {
		let mut current_server: Option<&CServer> = None;
		if let Some(servers) = &app.active_servers {
			for server in servers {
				if server.server_id.eq(&app.selected_serverid) {
					current_server = Some(server);
				}
			}
		}

		ui.horizontal(|ui| {
			ui.heading("Rooms");
			if let Some(server) = current_server {
				if server.is_admin && ui.button("+").clicked() {
					app.room_window_open = true;
				}
			}
		});
		
		ui.separator();


		if let Some(server) = current_server {
			for room in &server.rooms {
				if ui.add(SelectableLabel::new(room.roomid.eq(&app.selected_roomid), room.roomid.clone())).clicked() {
					if app.selected_roomid.eq(&room.roomid) {
						app.selected_roomid.clear();
					} else {
						app.selected_roomid = room.roomid.clone();
					}
				}
			}
		}
	});
}

pub fn messages(app: &mut RealmApp, ctx: &Context) {
	egui::CentralPanel::default().show(ctx, |ui| {
		ui.label(format!("Saved username: {:?}", app.saved_username));
		ui.label(format!("Saved token: {:?}", app.saved_token));
		ui.label(format!("Saved auth address: {:?}", app.saved_auth_address));
		
		ui.separator();

		if let Some(servers) = &app.active_servers {
			for server in servers {
				ui.heading(&server.server_id);
				ui.label(format!("{:?}", server));
			}
		}

		ui.separator();

		ui.label(format!("Current user: {:?}", app.current_user));
	});
}

pub fn modals(app: &mut RealmApp, ctx: &Context) {
	egui::Window::new("Signup")
		.open(&mut app.signup_window_open)
		.min_size((500.0, 200.0))
		.show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Server Address: ");
				ui.text_edit_singleline(&mut app.login_window_server_address);
			});

			ui.horizontal(|ui| {
				ui.label("Username: ");
				ui.text_edit_singleline(&mut app.login_window_username);
			});

			ui.horizontal(|ui| {
				ui.label("Email: ");
				ui.text_edit_singleline(&mut app.login_window_email);
			});

			if ui.button("Create Account").clicked() {
				let login_window_server_address = app.login_window_server_address.clone();
				let login_window_username = app.login_window_username.clone();
				let login_window_email = app.login_window_email.clone();
				let send_channel = app.login_start_channel.0.clone();

				let _handle = tokio::spawn(async move {
					let mut transport = tarpc::serde_transport::tcp::connect(login_window_server_address, Json::default);
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
					let result = client.create_account_flow(context::current(), login_window_username, login_window_email).await;

					match result {
						Ok(r) => send_channel.send(r).unwrap(),
						Err(_) => send_channel.send(Err(RPCError)).unwrap(),
					};
				});

				//ui.close_menu()
			}
		});

	egui::Window::new("Login")
		.open(&mut app.login_window_open)
		.min_size((500.0, 200.0))
		.show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Server Address: ");
				ui.text_edit_singleline(&mut app.login_window_server_address);
			});

			ui.horizontal(|ui| {
				ui.label("Username: ");
				ui.text_edit_singleline(&mut app.login_window_username);
			});

			if ui.button("Send Login Code").clicked() {
				let login_window_server_address = app.login_window_server_address.clone();
				let login_window_username = app.login_window_username.clone();
				let send_channel = app.login_start_channel.0.clone();

				let _handle = tokio::spawn(async move {
					let mut transport = tarpc::serde_transport::tcp::connect(login_window_server_address, Json::default);
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
					let result = client.create_login_flow(context::current(), Some(login_window_username), None).await;

					match result {
						Ok(r) => send_channel.send(r).unwrap(),
						Err(_) => send_channel.send(Err(RPCError)).unwrap(),
					};
				});

				//ui.close_menu()
			}
		});

	egui::Window::new("Auth Code")
		.open(&mut app.login_ready_for_code_input)
		.min_size((500.0, 200.0))
		.show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Code: ");
				if ui.text_edit_singleline(&mut app.login_window_code).changed() {
					let re = Regex::new(r"[^0-9]+").unwrap();
					app.login_window_code = re.replace_all(&app.login_window_code, "").to_string();
				}
			});

			if ui.button("Login").clicked() {
				let login_window_server_address = app.login_window_server_address.clone();
				let login_window_code = app.login_window_code.clone();
				let login_window_username = app.login_window_username.clone();
				let send_channel = app.login_ending_channel.0.clone();

				let _handle = tokio::spawn(async move {
					let mut transport = tarpc::serde_transport::tcp::connect(login_window_server_address, Json::default);
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
					let result = client.finish_login_flow(context::current(), login_window_username, login_window_code.parse::<u32>().unwrap()).await;

					match result {
						Ok(r) => {
							send_channel.send(r).unwrap();
						}
						Err(e) => {
							send_channel.send(Err(RPCError)).unwrap();
						}
					}
				});

				//ui.close_menu()
			}
		});
	
	egui::Window::new("Add Server")
		.open(&mut app.server_window_open)
		.min_size((500.0, 200.0))
		.show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Domain: ");
				ui.text_edit_singleline(&mut app.server_window_domain);
			});

			ui.horizontal(|ui| {
				ui.label("Port: ");
				if ui.text_edit_singleline(&mut app.server_window_port).changed() {
					let re = Regex::new(r"[^0-9]+").unwrap();
					app.login_window_code = re.replace_all(&app.server_window_port, "").to_string();
				}
			});


			if app.current_user.is_some() && ui.button("Add Server").clicked() {
				let domain = app.server_window_domain.clone();
				let port = app.server_window_port.clone();
				let auth_address = app.current_user.clone().unwrap().auth_address;
				let auth_username = app.current_user.clone().unwrap().username;
				let auth_token = app.current_user.clone().unwrap().token;
				let send_channel = app.add_server_channel.0.clone();

				let _handle = tokio::spawn(async move {
					let mut transport = tarpc::serde_transport::tcp::connect(auth_address, Json::default);
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
					let result = client.add_server(
						context::current(), auth_username, auth_token, domain.clone(), port.parse::<u16>().unwrap()).await;

					match result {
						Ok(r) => {
							match r {
								Ok(_) => {
									info!("Server added successfully!");
									send_channel.send(Ok(format!("{}:{}", domain, port))).unwrap();
								},
								Err(e) => error!("Error adding server: {:?}", e),
							}
						},
						Err(_) => { send_channel.send(Err(RPCError)).unwrap(); },
					};
				});
			}
		});
	
	egui::Window::new("Add Room")
		.open(&mut app.room_window_open)
		.min_size((500.0, 200.0))
		.show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Name: ");
				ui.text_edit_singleline(&mut app.room_window_name);
			});

			ui.checkbox(&mut app.room_window_admin_only_send, "Only admins can send");
			ui.checkbox(&mut app.room_window_admin_only_view, "Only admins can view");
			
			if ui.button("Add Room").clicked() {
				for server in app.active_servers.clone().unwrap() {
					if server.server_id.eq(&app.selected_serverid) {
						let token = app.current_user.as_ref().unwrap().token.clone();
						let roomid = app.room_window_name.clone();
						let admin_only_send = app.room_window_admin_only_send;
						let admin_only_view = app.room_window_admin_only_view;
						let userid = app.current_user.as_ref().unwrap().username.clone();
						let send_channel = app.add_room_channel.0.clone();
						let _handle = tokio::spawn(async move {
							let result = server.tarpc_conn.create_room(
								context::current(), 
								stoken(&token, &server.server_id, &server.domain, server.port),
								userid,
								Room {
									id: 0,
									roomid,
									admin_only_send,
									admin_only_view,
								}
							).await;
							
							match result {
								Ok(r) => {
									match r {
										Ok(_) => { send_channel.send(Ok(server)).unwrap(); }
										Err(e) => { send_channel.send(Err(e)).unwrap(); }
									}
								}
								Err(_) => {
									send_channel.send(Err(RPCError)).unwrap();
								}
							}
						});
					}
				}
			}
		});
}