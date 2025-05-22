use {reqwest::blocking::get, serde::Deserialize};

const API_BASE: &str = "https://meta.fabricmc.net/v2";

pub fn get_stable_fabric_for_version<T: Into<String>>(version: T) -> anyhow::Result<String> {
	let url = format!("{API_BASE}/versions/loader/{}", version.into());
	let response = get(&url)?.error_for_status()?;

	let versions_response: Vec<LoaderVersion> = response.json()?;
	Ok(format!("fabric-{}", versions_response[0].loader.version))
}

#[derive(Deserialize)]
struct LoaderVersion {
	loader: Loader,
	intermediary: Intermediary,
	#[serde(rename = "launcherMeta")]
	launcher_meta: LauncherMeta,
}

#[derive(Deserialize)]
struct Loader {
	separator: String,
	build: u8,
	maven: String,
	version: String,
	stable: bool,
}

#[derive(Deserialize)]
struct Intermediary {
	maven: String,
	version: String,
	stable: bool,
}

#[derive(Deserialize)]
struct LauncherMeta {
	version: u8,
	//min_java_version: u8,
	//libraries,
	//mainClass,
}
