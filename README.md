# Modpack Creator

## Usage

`modpack new <name>`

Creates a new modpack

`modpack init`

Creates a new modpack in the current directory

`modpack add <mod_name> -c <curseforge_link> -m <modrinth_link>`

Adds a new mod. -c and -m are optional, but you must use one or the other (or use -n if the mod has neither or must be downloaded manually)

`modpack check`

Check the compatibility of the mods included.
Checks in the order that they were added.

`modpack export -c|-m|-n`

Exports the mod to the exports folder

`-c`: Curseforge format
`-m`: Modrinth format (.mrpack)
`-n`: Modpackr format

## TODO

add tests that check for the following on important types:

    - Send
    - Sync

implement .mrpack exporting

implement .modpackr exporting and importing

get neoforge api working (will not work until then)

add an import command that extracts a .modpackr file(just a zip with config.toml, modpack.ron, and the mods folder(optional))

actually use config.toml
