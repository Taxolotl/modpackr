#![allow(dead_code)]

use {
	curseforge::CurseforgeMod,
	modrinth::ModrinthMod,
	serde::{Deserialize, Serialize},
	std::{
		collections::BTreeSet,
		fmt::Display,
		ops::{Add, AddAssign, Sub, SubAssign},
	},
};

pub mod curseforge;
pub mod fabric;
pub mod forge;
pub mod modrinth;
pub mod neoforge;
pub mod quilt;
pub mod util;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub struct Mod {
	pub name: String,

	// The ids of the mods
	pub curseforge: Option<CurseforgeMod>,
	pub modrinth: Option<ModrinthMod>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub struct ModVersions {
	pub fabric: BTreeSet<String>,
	pub forge: BTreeSet<String>,
	pub neo_forge: BTreeSet<String>,
	pub quilt: BTreeSet<String>,
}

impl AddAssign for ModVersions {
	fn add_assign(&mut self, rhs: Self) {
		*self = self.clone() + rhs;
	}
}

impl SubAssign for ModVersions {
	fn sub_assign(&mut self, rhs: Self) {
		*self = self.clone() - rhs;
	}
}

impl Add for ModVersions {
	type Output = ModVersions;

	fn add(self, rhs: Self) -> Self::Output {
		let mut fabric = BTreeSet::new();
		for version in self.fabric {
			fabric.insert(version);
		}
		for version in rhs.fabric {
			fabric.insert(version);
		}

		let mut forge = BTreeSet::new();
		for version in self.forge {
			forge.insert(version);
		}
		for version in rhs.forge {
			forge.insert(version);
		}

		let mut neo_forge = BTreeSet::new();
		for version in self.neo_forge {
			neo_forge.insert(version);
		}
		for version in rhs.neo_forge {
			neo_forge.insert(version);
		}

		let mut quilt = BTreeSet::new();
		for version in self.quilt {
			quilt.insert(version);
		}
		for version in rhs.quilt {
			quilt.insert(version);
		}

		Self {
			fabric,
			forge,
			neo_forge,
			quilt,
		}
	}
}

impl Sub for ModVersions {
	type Output = ModVersions;

	fn sub(self, rhs: Self) -> Self::Output {
		let fabric = self.fabric.intersection(&rhs.fabric).cloned().collect();
		let quilt = self.quilt.intersection(&rhs.quilt).cloned().collect();
		let forge = self.forge.intersection(&rhs.forge).cloned().collect();
		let neo_forge = self
			.neo_forge
			.intersection(&rhs.neo_forge)
			.cloned()
			.collect();

		Self {
			fabric,
			quilt,
			forge,
			neo_forge,
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub struct Modpack {
	pub name: String,
	pub version: String,
	pub minecraft_version: Option<String>,
	pub loader: Option<ModLoader>,
	pub author: String,
	pub mods: Vec<Mod>,
}

// TODO: Make this useful, and maybe add more stuff later
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub struct Config {
	export: ExportFormat,
	version: Option<String>,
	loader: Option<ModLoader>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub enum ExportFormat {
	#[default]
	Modrinth,
	Curseforge,
	Modpackr,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, PartialOrd, Eq, Ord)]
pub enum ModLoader {
	#[default]
	Fabric,
	Quilt,
	Forge,
	Neoforge,
}

impl Display for ModLoader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Fabric => "fabric",
				Self::Quilt => "quilt",
				Self::Forge => "forge",
				Self::Neoforge => "neoforge",
			}
		)
	}
}
