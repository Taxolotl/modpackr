use {
	crate::{ModLoader, ModVersions},
	anyhow::anyhow,
	chrono::{DateTime, Utc},
	reqwest::blocking::Client,
	serde::{Deserialize, Serialize},
	std::{collections::BTreeSet, time::Duration},
};
const API_BASE: &str = "https://api.curseforge.com/v1";

pub fn get_api_key() -> anyhow::Result<String> {
	Ok(std::env::var("CURSEFORGE_API_KEY")?)
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub struct CurseforgeMod {
	pub id: u32,
	pub name: String,
	pub summary: String,
	pub slug: String,
}

#[derive(Debug, Deserialize)]
struct GetModResponse {
	data: CurseforgeMod,
}

#[derive(Debug, Deserialize)]
struct GetSearchResponse {
	data: Vec<CurseforgeMod>,
}
pub struct CurseforgeClient {
	api_key: String,
	client: Client,
}

impl CurseforgeClient {
	pub fn new<T: Into<String>>(api_key: T) -> Self {
		Self {
			api_key: api_key.into(),
			client: Client::new(),
		}
	}

	pub fn get_mod(&self, mod_id: u32) -> anyhow::Result<CurseforgeMod> {
		std::thread::sleep(Duration::from_secs(1));
		let url = format!("{API_BASE}/mods/{}", mod_id);
		let res = self
			.client
			.get(&url)
			.header("x-api-key", &self.api_key)
			.send()?
			.error_for_status()?;

		let response: GetModResponse = res.json()?;
		Ok(response.data)
	}

	pub fn from_url<T: Into<String>>(&self, url: T) -> anyhow::Result<CurseforgeMod> {
		let url: String = url.into();
		let slug = url
			.trim_end_matches('/')
			.split('/')
			.last()
			.map(str::to_string)
			.ok_or(anyhow!("Error parsing url"))?;

		let search_result = self.search_slug(&slug)?;
		let mut data = None;
		for result in search_result.iter() {
			if result.slug == slug {
				data = Some(result.id);
				break;
			}
		}

		if let Some(id) = data {
			Ok(self.get_mod(id)?)
		} else {
			Err(anyhow!("Failed to find mod in search"))
		}
	}

	pub fn search_mod<T: Into<String>>(&self, search: T) -> anyhow::Result<Vec<CurseforgeMod>> {
		std::thread::sleep(Duration::from_secs(1));
		let url = format!(
			"{}/mods/search?gameId=432&searchFilter={}",
			API_BASE,
			search.into()
		);

		let response = self
			.client
			.get(&url)
			.header("x-api-key", &self.api_key)
			.send()?
			.error_for_status()?;

		let search_response: GetSearchResponse = response.json()?;

		Ok(search_response.data)
	}

	pub fn search_slug<T: Into<String>>(&self, slug: T) -> anyhow::Result<Vec<CurseforgeMod>> {
		std::thread::sleep(Duration::from_secs(1));
		let url = format!("{}/mods/search?gameId=432&slug={}", API_BASE, slug.into());

		let response = self
			.client
			.get(&url)
			.header("x-api-key", &self.api_key)
			.send()?
			.error_for_status()?;

		let search_response: GetSearchResponse = response.json()?;

		Ok(search_response.data)
	}

	pub fn get_versions(&self, id: u32) -> anyhow::Result<ModVersions> {
		std::thread::sleep(Duration::from_secs(1));
		let url = format!("{}/mods/{}/files", API_BASE, id);
		let resp = self
			.client
			.get(&url)
			.header("x-api-key", &self.api_key)
			.send()?
			.error_for_status()?;

		let json: serde_json::Value = resp.json()?;
		let new = Vec::new();
		let files = json["data"].as_array().unwrap_or(&new);

		let mut fabric = BTreeSet::new();
		let mut forge = BTreeSet::new();
		let mut neo_forge = BTreeSet::new();
		let mut quilt = BTreeSet::new();

		for file in files {
			let game_versions = file["gameVersions"]
				.as_array()
				.unwrap_or(&vec![])
				.iter()
				.filter_map(|v| v.as_str())
				.map(|s| s.to_lowercase())
				.collect::<Vec<_>>();

			let is_fabric = game_versions.iter().any(|v| v == "fabric");
			let is_forge = game_versions.iter().any(|v| v == "forge");
			let is_neoforge = game_versions.iter().any(|v| v == "neoforge");
			let is_quilt = game_versions.iter().any(|v| v == "quilt");

			let mc_versions = game_versions
				.iter()
				.filter(|v| {
					v.chars()
						.next()
						.map(|c| c.is_ascii_digit())
						.unwrap_or(false)
				})
				.cloned()
				.collect::<Vec<_>>();

			if is_fabric {
				fabric.extend(mc_versions.iter().cloned());
			}
			if is_forge {
				forge.extend(mc_versions.iter().cloned());
			}
			if is_neoforge {
				neo_forge.extend(mc_versions.iter().cloned());
			}
			if is_quilt {
				quilt.extend(mc_versions.iter().cloned());
			}
		}

		Ok(ModVersions {
			fabric: fabric.into_iter().collect(),
			forge: forge.into_iter().collect(),
			neo_forge: neo_forge.into_iter().collect(),
			quilt: quilt.into_iter().collect(),
		})
	}

	pub fn latest_stable(
		&self,
		id: u32,
		loader: ModLoader,
	) -> anyhow::Result<CurseforgeManifestFile> {
		let files = self.get_mod_files(id)?;

		let loader_name = match loader {
			ModLoader::Fabric => "Fabric",
			ModLoader::Quilt => "Quilt",
			ModLoader::Forge => "Forge",
			ModLoader::Neoforge => "Neoforge",
		}
		.to_owned();

		let mut stable_files = files
			.into_iter()
			.filter(|file| file.release_type <= 2 && file.game_versions.contains(&loader_name))
			.collect::<Vec<_>>();

		stable_files.sort_by(|a, b| b.file_date.cmp(&a.file_date));

		stable_files
			.first()
			.map(|file| CurseforgeManifestFile {
				project_id: id,
				file_id: file.id,
			})
			.ok_or(anyhow!(
				"No stable version found for {} with loader {}",
				id,
				loader
			))
	}

	pub fn get_mod_files(&self, id: u32) -> anyhow::Result<Vec<CurseforgeModFile>> {
		std::thread::sleep(Duration::from_secs(1));
		let url = format!("{API_BASE}/mods/{}/files", id);

		let response = self
			.client
			.get(&url)
			.header("x-api-key", self.api_key.clone())
			.send()?
			.error_for_status()?;

		let mod_files_response: CurseforgeModFilesResponse = response.json()?;

		Ok(mod_files_response.data)
	}
}

#[derive(Debug, Deserialize)]
pub struct CurseforgeModFile {
	pub id: u32,
	#[serde(rename = "fileName")]
	pub file_name: String,
	#[serde(rename = "releaseType")]
	pub release_type: u8, // 1=release, 2=beta, 3=alpha
	#[serde(rename = "fileDate")]
	pub file_date: DateTime<Utc>,
	#[serde(rename = "gameVersions")]
	pub game_versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeModFilesResponse {
	pub data: Vec<CurseforgeModFile>,
}

#[derive(Serialize)]
pub struct CurseforgeManifest {
	pub minecraft: CurseforgeMinecraftManifest,
	#[serde(rename = "manifestType")]
	pub manifest_type: String,
	#[serde(rename = "manifestVersion")]
	pub manifest_version: u8,
	pub name: String,
	pub version: String,
	pub author: String,
	pub files: Vec<CurseforgeManifestFile>,
	pub overrides: String,
}

#[derive(Serialize)]
pub struct CurseforgeManifestFile {
	#[serde(rename = "projectID")]
	pub project_id: u32,
	#[serde(rename = "fileID")]
	pub file_id: u32,
}

#[derive(Serialize)]
pub struct CurseforgeMinecraftManifest {
	pub version: String,
	#[serde(rename = "modLoaders")]
	pub mod_loaders: Vec<CurseforgeModLoaderEntry>,
}

#[derive(Serialize)]
pub struct CurseforgeModLoaderEntry {
	pub id: String,
	pub primary: bool,
}
