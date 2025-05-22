use {
	dotenv::dotenv,
	eframe::egui::{self, ComboBox},
	modpackr::{
		ExportFormat, Modpack,
		util::{add_mod, check, create_project_at_path, export, load_modpack},
	},
	rfd::FileDialog,
	std::{
		path::PathBuf,
		sync::{Arc, Mutex},
		thread::JoinHandle,
	},
};

fn main() -> Result<(), eframe::Error> {
	dotenv().ok();
	let options = eframe::NativeOptions::default();
	eframe::run_native(
		"Modpack GUI",
		options,
		Box::new(|_cc| Ok(Box::new(MyApp::default()))),
	)
}

#[derive(Default)]
struct MyApp {
	current_project: Option<Modpack>,
	project_path: Option<PathBuf>,
	status_log: Arc<Mutex<Vec<String>>>,
	backup_log: Vec<String>,

	add_mod_name: String,
	add_mod_use_modrinth: bool,
	add_mod_modrinth: String,
	add_mod_use_curseforge: bool,
	add_mod_curseforge: String,

	is_checking: bool,
	check_task: Option<JoinHandle<()>>,

	is_exporting: bool,
	export_task: Option<JoinHandle<()>>,
	export_format: ExportFormat,

	screen: Screen,
}

#[derive(Default)]
enum Screen {
	#[default]
	None,
	Open,
	Add,
	Export,
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let status_log = self.status_log.clone();

		egui::CentralPanel::default().show(ctx, |ui| {
			match self.screen {
				Screen::None => {
					ui.heading("No project selected. Open or create a new one");
					if ui.button("Create New Project").clicked() {
						if let Some(folder) = FileDialog::new().pick_folder() {
							let mut log = self.status_log.lock().expect("Error locking status log");
							match create_project_at_path(&folder) {
								Ok(_) => {
									log.push(format!(
										"Created new project at {}",
										folder.display()
									));
									self.project_path = Some(folder.clone());
									self.current_project =
										Some(load_modpack(&folder).expect("Error loading modpack"));
									self.screen = Screen::Open;
								},
								Err(e) => log.push(format!("Failed to create project: {}", e)),
							}
							drop(log);
						}
					}
					if ui.button("Open").clicked() {
						if let Some(folder) = FileDialog::new().pick_folder() {
							let mut log = self.status_log.lock().expect("Error locking status log");
							match load_modpack(&folder) {
								Ok(_) => {
									log.push(format!("Opened project {}", folder.display()));
									self.project_path = Some(folder.clone());
									self.current_project = Some(load_modpack(&folder).unwrap());
									self.screen = Screen::Open;
								},
								Err(e) => log.push(format!("Failed to open project: {}", e)),
							}
							drop(log);
						}
					}
					if ui.button("Import").clicked() {
						let mut log = self.status_log.lock().expect("Error locking status log");
						log.push("This is still being worked on...".into());
						drop(log);
					}
				},
				Screen::Open => {
					ui.heading(&self.current_project.as_ref().unwrap().name);
					ui.label("Currently included mods:");
					egui::ScrollArea::vertical().show(ui, |ui| {
						for line in &self.current_project.as_ref().unwrap().mods {
							ui.label(&line.name);
						}
					});
					if ui.button("Add a new mod").clicked() {
						self.screen = Screen::Add;
					}

					if !self.is_checking {
						if ui.button("Check Compatibility").clicked() {
							self.is_checking = true;

							let mut log = self.status_log.lock().expect("Error locking status log");
							log.push("Running check...".into());
							drop(log);

							let status_log = self.status_log.clone();
							let path = std::sync::Arc::new(self.project_path.clone().unwrap());

							self.check_task = Some(std::thread::spawn(move || {
								let mut log = status_log.lock().expect("Error locking status log");
								match check(&path) {
									Ok((result, manual_mods)) => {
										if !manual_mods.is_empty() {
											log.push(format!(
												"{} mods have no provider and may require manually inputting them",
												manual_mods.len()
											));
											for m in manual_mods {
												log.push(format!("\t{m}"));
											}
										}

										if let Some((loader, version)) = result {
											log.push(format!(
												"Found a compatible version and loader!\nLoader: {loader}\nVersion: {version}"
											));
										} else {
											log.push("No compatible version found".into());
										}
									},
									Err(e) => log.push(format!("Check failed: {}", e)),
								}
							}));
						}
					} else {
						ui.add_enabled(
							false,
							egui::Button::new("Check Compatibility (Running...)"),
						);

						if let Some(handle) = &self.check_task {
							if handle.is_finished() {
								let _ = self.check_task.take().unwrap().join();
								self.is_checking = false;
							}
						}
					}

					if ui.button("Export").clicked() {
						self.screen = Screen::Export;
					}

					if ui.button("Close Project").clicked() {
						self.screen = Screen::None;
						self.current_project = None;
						self.project_path = None;
					}
				},
				Screen::Add => {
					ui.label("Name:");
					ui.text_edit_singleline(&mut self.add_mod_name);

					ui.label("Modrinth Link:");
					ui.text_edit_singleline(&mut self.add_mod_modrinth);
					ui.checkbox(&mut self.add_mod_use_modrinth, "Use modrinth?");

					ui.label("CurseForge Link:");
					ui.text_edit_singleline(&mut self.add_mod_curseforge);
					ui.checkbox(&mut self.add_mod_use_curseforge, "Use curseforge?");

					if ui.button("Add mod").clicked() {
						let name = self.add_mod_name.trim();
						let modrinth = if self.add_mod_use_modrinth {
							Some(self.add_mod_modrinth.trim())
						} else {
							None
						};
						let curseforge = if self.add_mod_use_curseforge {
							Some(self.add_mod_curseforge.trim())
						} else {
							None
						};

						let mut log = self.status_log.lock().expect("Error locking status log");

						if name.is_empty() {
							log.push("Error: Mod name is required.".into());
						} else {
							match add_mod(
								self.project_path.as_ref().unwrap(),
								name,
								curseforge,
								modrinth,
								!self.add_mod_use_modrinth && !self.add_mod_use_curseforge,
							) {
								Ok(new_mod) => {
									log.push(format!("Successfully added {name}!"));
									self.current_project.as_mut().unwrap().mods.push(new_mod);
									self.screen = Screen::Open;
								},
								Err(e) => log.push(format!("Failed to add mod '{}': {}", name, e)),
							}
						}
						drop(log);
					}

					ui.separator();

					if ui.button("Return (Will NOT save)").clicked() {
						self.screen = Screen::Open;
					}
				},
				Screen::Export => {
					ComboBox::from_label("Export Format")
						.selected_text(format!("{:?}", self.export_format))
						.show_ui(ui, |ui| {
							ui.selectable_value(
								&mut self.export_format,
								ExportFormat::Modrinth,
								"Modrinth (.mrpack)",
							);
							ui.selectable_value(
								&mut self.export_format,
								ExportFormat::Curseforge,
								"CurseForge",
							);
							ui.selectable_value(
								&mut self.export_format,
								ExportFormat::Modpackr,
								"Modpackr",
							);
						});

					if !self.is_exporting {
						if ui.button("Export").clicked() {
							self.is_exporting = true;

							let mut log = self.status_log.lock().expect("Error locking status log");
							log.push("Exporting".into());
							drop(log);

							let status_log = self.status_log.clone();
							let path = Arc::new(self.project_path.clone().unwrap());
							let export_format = self.export_format.clone();

							self.export_task = Some(std::thread::spawn(move || {
								let mut log = status_log.lock().expect("Error locking status log");
								if let Err(e) = export(
									&path,
									export_format == ExportFormat::Curseforge,
									export_format == ExportFormat::Modrinth,
									export_format == ExportFormat::Modpackr,
								) {
									log.push(format!("Failed to export modpack: {e}"));
								} else {
									log.push("Successfully exported the modpack!".into());
								}
								drop(log);
							}));
						}
					} else {
						ui.add_enabled(false, egui::Button::new("Exporting..."));

						if let Some(handle) = &self.export_task {
							if handle.is_finished() {
								let _ = self.export_task.take().unwrap().join();
								self.is_exporting = false;
							}
						}
					}

					if ui.button("Return").clicked() {
						self.screen = Screen::Open;
					}
				},
			}

			ui.separator();
			ui.label("Status Log:");

			ui.push_id("Status Log", |ui| {
				egui::ScrollArea::vertical().show(ui, |ui| {
					if let Ok(log) = status_log.try_lock() {
						self.backup_log = log.clone();
						for line in log.iter() {
							ui.label(line);
						}
					} else {
						for line in self.backup_log.iter() {
							ui.label(line);
						}
					}
				});
			});
		});
	}
}
