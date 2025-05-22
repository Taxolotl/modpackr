use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "modpackr", version, author, about = "Modpack management tool")]
pub struct ModpackrCli {
	#[command(subcommand)]
	pub command: ModpackrCommand,
}

#[derive(Subcommand)]
pub enum ModpackrCommand {
	Init,
	New {
		name: String,
	},
	Add {
		mod_name: String,

		#[arg(short = 'c', long)]
		curseforge: Option<String>,

		#[arg(short = 'm', long)]
		modrinth: Option<String>,

		#[arg(short = 'n', long)]
		manual: bool,
	},
	Check,
	Export {
		#[arg(short = 'c', long)]
		curseforge: bool,

		#[arg(short = 'm', long)]
		modrinth: bool,

		#[arg(short = 'n', long)]
		neither: bool,
	},
}
