use {
	crate::ModVersions,
	anyhow::anyhow,
	reqwest::blocking::get,
	serde::{Deserialize, Serialize},
	std::collections::BTreeSet,
};
const API_BASE: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub struct ModrinthMod {
	pub id: String,
	pub title: String,
	pub description: String,
	pub slug: String,
}

pub fn get_mod_from_modrinth<T: Into<String>>(mod_id: T) -> anyhow::Result<ModrinthMod> {
	let url = format!("{API_BASE}/project/{}", mod_id.into());
	let res = get(&url)?.error_for_status()?;

	let response: ModrinthMod = res.json()?;

	Ok(response)
}

pub fn get_modrinth_mod_from_url<T: Into<String>>(url: T) -> anyhow::Result<ModrinthMod> {
	let url: String = url.into();

	let slug = if let Some(slug) = url
		.trim_end_matches('/')
		.split('/')
		.last()
		.map(str::to_string)
	{
		slug
	} else {
		return Err(anyhow!("Error parsing url"));
	};

	get_mod_from_modrinth(slug)
}

pub fn get_versions_from_modrinth(id: String) -> anyhow::Result<ModVersions> {
	let url = format!("{API_BASE}/project/{}/version", id);
	let resp = get(&url)?.error_for_status()?;

	let json: serde_json::Value = resp.json()?;
	let default = Vec::new();
	let versions = json.as_array().unwrap_or(&default);

	let mut fabric = BTreeSet::new();
	let mut forge = BTreeSet::new();
	let mut neo_forge = BTreeSet::new();
	let mut quilt = BTreeSet::new();

	for v in versions {
		let loaders = v["loaders"]
			.as_array()
			.unwrap_or(&vec![])
			.iter()
			.filter_map(|l| l.as_str())
			.map(|s| s.to_lowercase())
			.collect::<Vec<_>>();

		let game_versions = v["game_versions"]
			.as_array()
			.unwrap_or(&vec![])
			.iter()
			.filter_map(|g| g.as_str())
			.map(|s| s.to_string())
			.collect::<Vec<_>>();

		for loader in loaders {
			match loader.as_str() {
				"fabric" => fabric.extend(game_versions.iter().cloned()),
				"forge" => forge.extend(game_versions.iter().cloned()),
				"neoforge" => neo_forge.extend(game_versions.iter().cloned()),
				"quilt" => quilt.extend(game_versions.iter().cloned()),
				_ => {},
			}
		}
	}

	Ok(ModVersions {
		fabric,
		forge,
		neo_forge,
		quilt,
	})
}
