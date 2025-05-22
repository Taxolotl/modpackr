use {
	crate::{
		Config, Mod, ModLoader, ModVersions, Modpack,
		curseforge::{
			CurseforgeClient, CurseforgeManifest, CurseforgeMinecraftManifest,
			CurseforgeModLoaderEntry, get_api_key,
		},
		fabric::get_stable_fabric_for_version,
		forge::get_latest_forge_version,
		modrinth::{get_modrinth_mod_from_url, get_versions_from_modrinth},
		neoforge::get_latest_neoforge_version,
		quilt::get_latest_quilt_for_version,
	},
	anyhow::anyhow,
	std::{collections::BTreeMap, env::current_dir, fs, io::Write, path::Path},
	zip::{
		ZipWriter,
		write::{ExtendedFileOptions, FileOptions},
	},
};

pub fn get_loader_version<T: Into<String>>(
	loader: ModLoader,
	version: T,
) -> anyhow::Result<String> {
	match loader {
		ModLoader::Fabric => get_stable_fabric_for_version(version),
		ModLoader::Quilt => get_latest_quilt_for_version(version),
		ModLoader::Forge => get_latest_forge_version(version),
		ModLoader::Neoforge => get_latest_neoforge_version(version),
	}
}

pub fn find_compatible(
	versions: &[ModVersions],
	force_loader: Option<ModLoader>,
	force_version: Option<&str>,
) -> Option<(ModLoader, String)> {
	if versions.is_empty() {
		return None;
	}

	let mut intersection = versions[0].clone();
	for v in &versions[1..] {
		intersection -= v.clone();
	}

	let loader_map = BTreeMap::from([
		(ModLoader::Quilt, &intersection.quilt),
		(ModLoader::Fabric, &intersection.fabric),
		(ModLoader::Neoforge, &intersection.neo_forge),
		(ModLoader::Forge, &intersection.forge),
	]);

	for (loader, versions) in loader_map {
		if versions.is_empty() {
			continue;
		}

		if let Some(ref force_loader) = force_loader {
			if loader != *force_loader {
				continue;
			}
		}

		if let Some(force_version) = force_version {
			if versions.contains(&force_version.to_owned()) {
				return Some((loader, force_version.to_owned()));
			}
		} else {
			return Some((loader, versions.last().unwrap().to_string()));
		}
	}

	None
}

pub fn load_versions(project_dir: &Path) -> anyhow::Result<Vec<ModVersions>> {
	let mut results = Vec::new();

	for entry in fs::read_dir(project_dir.join("mods"))? {
		let entry = entry.unwrap();
		let path = entry.path();

		if path.extension().map_or(false, |ext| ext == "ron") {
			let contents = fs::read_to_string(&path)?;
			let versions: ModVersions = ron::from_str(&contents)?;
			results.push(versions);
		}
	}

	Ok(results)
}

pub fn load_config(project_dir: &Path) -> anyhow::Result<Config> {
	let toml = project_dir.join("config.toml");

	if toml.exists() {
		let contents = fs::read_to_string(toml)?;
		Ok(toml::from_str(&contents)?)
	} else {
		Err(anyhow!(
			"This directory has not been initialized, use `modpack init` to initialize or use `modpack new <name>` to create a new modpack with the name specified"
		))
	}
}

pub fn load_modpack(project_dir: &Path) -> anyhow::Result<Modpack> {
	let ron = project_dir.join("modpack.ron");

	if ron.exists() {
		let contents = fs::read_to_string(ron)?;
		Ok(ron::from_str(&contents)?)
	} else {
		Err(anyhow!(
			"modpack.ron was not found, use `modpack init` to initialize this directory or use `modpack new <name>` to create a new modpack with the name specified"
		))
	}
}

pub fn update_modpack(project_dir: &Path, pack: Modpack) -> anyhow::Result<()> {
	let ron = ron::ser::to_string_pretty(&pack, ron::ser::PrettyConfig::default())?;

	Ok(fs::write(project_dir.join("modpack.ron"), ron)?)
}

pub fn create_new_project(name: &str) -> anyhow::Result<()> {
	let path = current_dir()?.join(name);
	create_project_at_path(&path)
}

pub fn initialize_project() -> anyhow::Result<()> {
	create_project_at_path(&current_dir()?)
}

pub fn create_project_at_path(path: &Path) -> anyhow::Result<()> {
	fs::create_dir_all(path)?;
	fs::create_dir_all(path.join("mods"))?;

	let modpack = Modpack {
		name: path
			.file_name()
			.and_then(|os_str| os_str.to_str())
			.map(|s| s.to_string())
			.ok_or(anyhow!("Path terminated in .."))?,
		version: "1.0.0".to_owned(),
		..Default::default()
	};

	update_modpack(path, modpack)?;

	let config = Config::default();
	let toml = toml::ser::to_string_pretty(&config)?;
	fs::write(path.join("config.toml"), toml)?;

	Ok(())
}

pub fn add_mod<T: Into<String>, U: Into<String>, V: Into<String>>(
	project_dir: &Path,
	mod_name: T,
	curseforge: Option<U>,
	modrinth: Option<V>,
	manual: bool,
) -> anyhow::Result<Mod> {
	if curseforge.is_some() && manual ||
		modrinth.is_some() && manual ||
		(curseforge.is_none() && modrinth.is_none() && !manual)
	{
		return Err(anyhow!(
			"You must specify what kind of mod this is. to use a curseforge link, use -c <link>, for modrinth -m <link> and if it is on neither of those, specify it as a manual mod with -n"
		));
	}

	let mut modpack = load_modpack(project_dir)?;

	let curseforge = if let Some(curseforge) = curseforge {
		let curseforge_client = CurseforgeClient::new(get_api_key()?);
		let curseforge_mod_data = curseforge_client.from_url(curseforge.into())?;

		Some(curseforge_mod_data)
	} else {
		None
	};

	let modrinth = if let Some(modrinth) = modrinth {
		let modrinth_mod_data = get_modrinth_mod_from_url(modrinth.into())?;

		Some(modrinth_mod_data)
	} else {
		None
	};

	let name = mod_name.into();

	let mod_data = Mod {
		name: name.clone(),
		curseforge,
		modrinth,
	};

	modpack.mods.push(mod_data.clone());

	update_modpack(project_dir, modpack)?;

	Ok(mod_data)
}

pub fn check(project_dir: &Path) -> anyhow::Result<(Option<(ModLoader, String)>, Vec<String>)> {
	let _config = load_config(project_dir)?;
	let mut modpack = load_modpack(project_dir)?;
	let curseforge_client = CurseforgeClient::new(get_api_key()?);
	let mut manual_mods = Vec::new();
	for entry in modpack.mods.iter() {
		if entry.modrinth.is_none() && entry.curseforge.is_none() {
			manual_mods.push(entry.name.clone());
			continue;
		}

		let curseforge_versions = if entry.curseforge.is_some() {
			curseforge_client.get_versions(entry.curseforge.clone().unwrap().id)?
		} else {
			ModVersions::default()
		};

		let modrinth_versions = if entry.modrinth.is_some() {
			get_versions_from_modrinth(entry.modrinth.clone().unwrap().id)?
		} else {
			ModVersions::default()
		};

		let versions = modrinth_versions + curseforge_versions;

		let ron_string = ron::ser::to_string_pretty(&versions, ron::ser::PrettyConfig::default())?;
		fs::write(
			project_dir.join("mods").join(format!("{}.ron", entry.name)),
			ron_string,
		)?;
	}

	let versions_vec = load_versions(project_dir)?;
	let result = find_compatible(&versions_vec, None, None);
	if let Some((loader, version)) = result {
		modpack.loader = Some(loader.clone());
		modpack.minecraft_version = Some(version.clone());

		update_modpack(project_dir, modpack)?;
		Ok((Some((loader, version)), manual_mods))
	} else {
		Ok((None, manual_mods))
	}
}

pub fn export(
	project_dir: &Path,
	curseforge: bool,
	modrinth: bool,
	neither: bool,
) -> anyhow::Result<()> {
	let sum = curseforge as u8 + modrinth as u8 + neither as u8;
	let modrinth = if sum == 0 { true } else { modrinth };
	if sum != 1 {
		return Err(anyhow!(
			"You must specify either zero or one of -c, -m (the default if you specify none), or -n"
		));
	}

	let modpack = load_modpack(project_dir)?;

	let modlist = modpack.mods.clone();
	let loader = modpack.loader.clone();
	let minecraft_version = modpack.minecraft_version.clone();
	let name = modpack.name;
	let author = modpack.author;
	let version = modpack.version;

	let mut both_provider_mods = Vec::new();
	let mut modrinth_mods = Vec::new();
	let mut curseforge_mods = Vec::new();
	let mut manual_mods = Vec::new();

	for m in modlist.iter() {
		match (&m.modrinth, &m.curseforge) {
			(Some(_), Some(_)) => {
				both_provider_mods.push(m);
			},
			(Some(_), None) => {
				modrinth_mods.push(m);
			},
			(None, Some(_)) => {
				curseforge_mods.push(m);
			},
			_ => {
				manual_mods.push(m);
			},
		}
	}

	if !manual_mods.is_empty() {
		println!(
			"[WARN] {} mods must be manually installed due to a lack of a provider",
			manual_mods.len()
		);
	}
	for m in manual_mods.iter() {
		println!("\t{}", m.name);
	}

	if curseforge {
		if modrinth_mods.len() != 0 {
			println!(
				"[WARN] There are some mods that are only available on modrinth and must be installed manually: "
			);
			for m in modrinth_mods {
				println!(
					"\t {} https://modrinth.com/mod/{}",
					m.name,
					m.modrinth.as_ref().unwrap().id
				);
			}
		}

		let mod_loader = CurseforgeModLoaderEntry {
			id: get_loader_version(
				loader
					.clone()
					.ok_or(anyhow!("No loader specified in modpack.ron"))?,
				minecraft_version
					.clone()
					.ok_or(anyhow!("No minecraft version specified in modpack.ron"))?,
			)?,
			primary: true,
		};

		let minecraft_manifest = CurseforgeMinecraftManifest {
			version: minecraft_version
				.clone()
				.ok_or(anyhow!("modpack.ron did not contain a version"))?,
			mod_loaders: vec![mod_loader],
		};
		let curseforge = CurseforgeClient::new(get_api_key()?);

		let mods = curseforge_mods
			.into_iter()
			.chain(both_provider_mods)
			.map(|m| {
				curseforge.latest_stable(
					m.curseforge.as_ref().unwrap().id,
					loader
						.clone()
						.ok_or(anyhow!("No loader specified in modpack.ron"))?,
				)
			})
			.collect::<anyhow::Result<Vec<_>>>()?;

		let manifest = CurseforgeManifest {
			minecraft: minecraft_manifest,
			manifest_type: "minecraftModpack".to_owned(),
			manifest_version: 1,
			name: name.clone(),
			version: version.clone(),
			author,
			files: mods,
			overrides: "overrides".to_owned(),
		};

		let contents = serde_json::ser::to_string_pretty(&manifest)?;

		let output_path = project_dir.join("export");
		fs::create_dir_all(&output_path)?;

		let file =
			fs::File::create(output_path.join(format!("{}-{}-curseforge.zip", name, version)))?;

		let mut zip = ZipWriter::new(file);

		let options: FileOptions<'_, ExtendedFileOptions> = FileOptions::default()
			.compression_method(zip::CompressionMethod::Stored)
			.unix_permissions(0o644);

		zip.start_file("manifest.json", options)?;
		zip.write_all(contents.as_bytes())?;

		zip.finish()?;
	} else if modrinth {
		return Err(anyhow!("Still working on this, try modpackr or curseforge"));
	} else {
		return Err(anyhow!("Still working on this, try modrinth or curseforge"));
	}

	Ok(())
}
