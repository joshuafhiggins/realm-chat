use egui::{Context, SelectableLabel};
use crate::app::TemplateApp;

pub fn top_panel(ctx: &Context) {
	egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
		// The top panel is often a good place for a menu bar:

		egui::menu::bar(ui, |ui| {
			// NOTE: no File->Quit on web pages!
			let is_web = cfg!(target_arch = "wasm32");
			if !is_web {
				ui.menu_button("File", |ui| {
					if ui.button("Quit").clicked() {
						ctx.send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});
				ui.add_space(16.0);
			}

			egui::widgets::global_dark_light_mode_buttons(ui);
		});
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