param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][ValidateSet('base', 'describe', 'domain', 'compose')][string]$Level
)

$ErrorActionPreference = 'Stop'

$directory = "crates/plugins/$Level/nightshade-$Name"
if (Test-Path $directory) {
    throw "$directory already exists"
}

$pascal = (($Name -split '-') | ForEach-Object { $_.Substring(0, 1).ToUpper() + $_.Substring(1) }) -join ''

New-Item -ItemType Directory -Force "$directory/src/systems" | Out-Null

$manifest = @"
[package]
name = "nightshade-$Name"
version.workspace = true
edition.workspace = true
license.workspace = true
publish = false

[dependencies]
nightshade = { workspace = true, optional = true }
nightshade-platform = { path = "../../describe/nightshade-platform", default-features = false, features = [
    "contract",
], optional = true }

[features]
default = ["plugin"]
contract = []
plugin = ["contract", "dep:nightshade", "dep:nightshade-platform"]
"@

$library = @"
#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
pub mod components;
#[cfg(feature = "plugin")]
pub mod data;
#[cfg(feature = "plugin")]
pub mod plugin;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "plugin")]
pub use plugin::${pascal}Plugin;
"@

$resources = @"
#[derive(Default)]
pub struct ${pascal}Settings {}
"@

$plugin = @"
use crate::resources::${pascal}Settings;
use nightshade::app::App;
use nightshade::app::Stage;
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct ${pascal}Plugin {
    pub settings: ${pascal}Settings,
}

impl Plugin for ${pascal}Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(${pascal}Settings::default());
        app.add_system(Stage::Update, crate::systems::tick::tick);
    }
}
"@

$tick = @"
use crate::resources::${pascal}Settings;
use nightshade::prelude::*;

pub fn tick(_settings: Res<${pascal}Settings>) {}
"@

Set-Content "$directory/Cargo.toml" -Encoding utf8 -Value $manifest
Set-Content "$directory/src/lib.rs" -Encoding utf8 -Value $library
Set-Content "$directory/src/resources.rs" -Encoding utf8 -Value $resources
Set-Content "$directory/src/plugin.rs" -Encoding utf8 -Value $plugin
Set-Content "$directory/src/systems.rs" -Encoding utf8 -Value "pub mod tick;"
Set-Content "$directory/src/systems/tick.rs" -Encoding utf8 -Value $tick
foreach ($band in 'contract', 'components', 'data', 'commands', 'queries') {
    Set-Content "$directory/src/$band.rs" -Encoding utf8 -Value ''
}

Write-Host "wrote $directory"
Write-Host "the bands are there and empty, so fill components, resources, data, queries, commands and systems"
Write-Host "then run: just check-crate nightshade-$Name"
