use egui::{Context, SelectableLabel};
use tarpc::context;
use tarpc::tokio_serde::formats::Json;
use realm_auth::types::RealmAuthClient;
use realm_shared::types::ErrorCode::RPCError;
use regex::Regex;
use crate::app::TemplateApp;

pub fn top_panel(app: &mut TemplateApp, ctx: &Context) {
	egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
		egui::menu::bar(ui, |ui| {
			ui.menu_button("File", |ui| {
				if ui.button("Sign Up").clicked() {
					app.signup_window_open = true;
				}

				if ui.button("Login").clicked() {
					app.login_window_open = true;
				}

				if ui.button("Logout").clicked() {
					// TODO: Logout
				}

				if ui.button("Quit").clicked() {
					ctx.send_viewport_cmd(egui::ViewportCommand::Close);
				}
			});
			ui.add_space(16.0);

			egui::widgets::global_dark_light_mode_buttons(ui);
		});
	});

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

				ui.close_menu()
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

				ui.close_menu()
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

				ui.close_menu()
			}
		});
}

pub fn servers(app: &mut TemplateApp, ctx: &Context) {
	egui::SidePanel::left("servers").show(ctx, |ui| {
		ui.heading("Servers");
		ui.separator();

		if ui.add(SelectableLabel::new(app.selected, "server")).clicked() {
			app.selected = !app.selected;
		}
	});
}

pub fn rooms(app: &mut TemplateApp, ctx: &Context) {
	egui::SidePanel::left("rooms").show(ctx, |ui| {
		ui.heading("Rooms");
		ui.separator();

		if ui.add(SelectableLabel::new(app.selected, "room")).clicked() {
			app.selected = !app.selected;
		}
	});
}

pub fn messages(app: &mut TemplateApp, ctx: &Context) {
	egui::CentralPanel::default().show(ctx, |ui| {
		// The central panel the region left after adding TopPanel's and SidePanel's
		ui.heading("eframe template");

		ui.horizontal(|ui| {
			ui.label("Write something: ");
			ui.text_edit_singleline(&mut app.label);
		});

		ui.add(egui::Slider::new(&mut app.value, 0.0..=10.0).text("value"));
		if ui.button("Increment").clicked() {
			app.value += 1.0;
		}

		ui.separator();
	});
}