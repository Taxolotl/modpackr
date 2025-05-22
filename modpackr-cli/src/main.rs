use {
	clap::Parser,
	modpackr::util::*,
	std::env::current_dir,
	util::{ModpackrCli, ModpackrCommand},
};

mod util;

fn main() -> anyhow::Result<()> {
	dotenv::dotenv()?;
	let cli = ModpackrCli::parse();

	match cli.command {
		ModpackrCommand::New { name } => {
			if let Err(e) = create_new_project(name.as_str()) {
				eprintln!("Failed to create modpack: {e}");
				Err(e)
			} else {
				println!("Successfully created a new project");
				Ok(())
			}
		},
		ModpackrCommand::Init => {
			if let Err(e) = initialize_project() {
				eprintln!("Failed to initialize project: {e}");
				Err(e)
			} else {
				println!("Successfully initialized project");
				Ok(())
			}
		},
		ModpackrCommand::Add {
			mod_name,
			curseforge,
			modrinth,
			manual,
		} => {
			if let Err(e) = add_mod(&current_dir()?, &mod_name, curseforge, modrinth, manual) {
				eprintln!("Failed to add mod {}: {e}", mod_name);
				Err(e)
			} else {
				println!("Successfully added mod {}", mod_name);
				Ok(())
			}
		},
		ModpackrCommand::Check => {
			let (result, manual_mods) = check(&current_dir()?)?;

			if !manual_mods.is_empty() {
				println!(
					"{} mods have no provider and must be manually added to the mods folder",
					manual_mods.len()
				);
				for m in manual_mods {
					println!("\t{m}")
				}
			}

			if let Some((loader, version)) = result {
				println!("Found a compatible version!\nLoader: {loader}\nVersion: {version}");
			} else {
				println!("Failed to find a compatible version");
			}
			Ok(())
		},
		ModpackrCommand::Export {
			curseforge,
			modrinth,
			neither,
		} => {
			if let Err(e) = export(&current_dir()?, curseforge, modrinth, neither) {
				eprintln!("Failed to export modpack: {e}");
				Err(e)
			} else {
				println!("Successfully exported modpack");
				Ok(())
			}
		},
	}
}
