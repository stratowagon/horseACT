# horseACT

horseACT is a hachimi plugin to dump data at runtime.

## Installation

1. Install hachimi

Download and install Hachimi Edge from:

`https://github.com/kairusds/Hachimi-Edge/releases/latest`

2. Download `horseACT.dll`

Grab the latest `horseACT.dll` from:

`https://github.com/ayaliz/horseACT/releases/latest`

Place it in your game folder root.

3. Enable the plugin in hachimi

Open `hachimi/config.json` in your game folder and add `horseACT.dll` to `load_libraries`:

```json
{
  "load_libraries": [
    "horseACT.dll"
  ]
}
```

4. Launch the game once

horseACT will create `hachimi/horseACTConfig.json`. Race files are saved under `Saved races` in your Documents folder by default.

## What horseACT saves

- Room Match races
- Champions Meeting races
- Practice Room races
- Career races, unless disabled in config
- Team Trials race results, unless disabled in config

## Configuration

After first launch, `hachimi/horseACTConfig.json` will contain:

```json
{
  "outputPath": "%USERPROFILE%\\Documents",
  "apiKey": "",
  "serverUrl": "",
  "fieldBlacklist": [
    "_ownerViewerId",
    "_viewerId",
    "owner_viewer_id",
    "viewer_id",
    "<SimData>k__BackingField",
    "<SimReader>k__BackingField",
    "CreateTime",
    "succession_history_array"
  ],
  "saveCareerRaces": true,
  "saveTTRaces": true
}
```

### `outputPath`

Base folder where the `Saved races` directory will be created.

### `apiKey`

API key for use with remote server.

### `serverUrl`

Base URL for use with remote server.

### `fieldBlacklist`

Field names that should be omitted from dumped race data. The defaults remove identifiable or redundant fields and help keep saved files smaller.

### `saveCareerRaces`

If `true`, Career races are saved locally. Set this to `false` if you only want room-match style data and do not want Career race files written to disk.

### `saveTTRaces`

If `true`, Team Trials race result responses are saved locally. Set this to `false` to skip writing Team Trials output files.
