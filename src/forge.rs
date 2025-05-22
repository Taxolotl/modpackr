use {
	anyhow::anyhow,
	quick_xml::{Reader, events::Event},
	reqwest::blocking::get,
};

pub fn get_latest_forge_version<T: Into<String>>(mc_version: T) -> anyhow::Result<String> {
	let mc_version = mc_version.into();
	let url = "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
	let response = get(url)?.text()?;

	let mut reader = Reader::from_str(&response);
	reader.trim_text(true);

	let mut versions = Vec::new();
	let mut buf = Vec::new();

	while let Ok(event) = reader.read_event_into(&mut buf) {
		if let Event::Text(e) = event {
			let text = e.unescape().unwrap_or_default().to_string();
			if text.starts_with(&format!("{}-", mc_version)) &&
				!text.contains("alpha") &&
				!text.contains("beta") &&
				!text.contains("rc")
			{
				versions.push(text);
			}
		} else if event == Event::Eof {
			break;
		}
		buf.clear();
	}

	versions.sort_by(|a, b| {
		let aforge = a.split('-').nth(1).unwrap_or("");
		let bforge = b.split('-').nth(1).unwrap_or("");
		natord::compare(aforge, bforge).reverse()
	});

	versions
		.first()
		.cloned()
		.map(|s| format!("forge{}", &s[mc_version.len()..]))
		.ok_or(anyhow!("No stable versions found in {}", url))
}
