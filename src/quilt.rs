use {reqwest::blocking::get, serde::Deserialize};

const API_BASE: &str = "https://meta.quiltmc.org/v3";

pub fn get_latest_quilt_for_version<T: Into<String>>(version: T) -> anyhow::Result<String> {
	let version: String = version.into();

	let url = format!("{API_BASE}/versions/loader/{}", version);
	let response = get(&url)?.error_for_status()?;

	let versions_response: Vec<LoaderVersion> = response.json()?;

	Ok(format!("quilt-{}", versions_response[0].loader.version))
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
}

#[derive(Deserialize)]
struct Intermediary {
	maven: String,
	version: String,
}

#[derive(Deserialize)]
struct LauncherMeta {
	version: u8,
	//min_java_version: u8,
	//libraries,
	//mainClass,
}
