# Modpack Creator

## Usage

`modpack new <name>`
Creates a new modpack

`modpack add <mod_name> -c <curseforge_link> -m <modrinth_link>`
Adds a new mod. -c and -m are optional, but you must use one or the other (or use -n if the mod has neither or must be downloaded manually)

`modpack check`
Check the compatibility of the mods included.
Checks in the order that they were added.

`modpack export`
Exports the mod. Defaults to modrinth if all mods are found there, however if curseforge is required (or specified with `modpack export -c`).
If there are mods that are exclusive to modrinth or curseforge when the rest requires another <!TODO>Figure out what to do in this situation<!/TODO>

## TODO

add tests that check for the following on important types:
    - Send
    - Sync
add an import command that extracts a .modpackr file(just a zip with config.toml, modpack.ron, and the mods folder(optional))
