set shell := ["pwsh", "-NoProfile", "-Command"]
set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]
export CARGO_INCREMENTAL := "0"
export RUST_LOG := "info,wgpu_core=off"
export RUST_BACKTRACE := "1"
export_apps := "tower_defense hairpin dsl"

@just:
    just --list

build:
    cargo build -r

build-book:
    trunk build --release --config apps/book/Trunk.toml --dist (Join-Path (Get-Location) 'book')

check:
    cargo check --all --all-targets
    $ErrorActionPreference = 'Stop'; $names = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages.name; foreach ($name in $names) { cargo fmt -p $name -- --check; if ($LASTEXITCODE -ne 0) { throw "rustfmt failed for $name" } }

clean-cache:
    @$ErrorActionPreference = 'Stop'; foreach ($directory in @('target/debug/incremental','target/release/incremental','target/wasm32-unknown-unknown/debug/incremental','target/wasm32-unknown-unknown/release/incremental')) { if (Test-Path $directory) { Write-Host "removing $directory"; Remove-Item -Recurse -Force $directory } }

docs $project="nightshade":
    cargo doc --open -p {{project}}

fix:
    cargo clippy --all --tests --fix

format:
    $ErrorActionPreference = 'Stop'; $names = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages.name; foreach ($name in $names) { cargo fmt -p $name; if ($LASTEXITCODE -ne 0) { throw "rustfmt failed for $name" } }

[linux]
install-debian-deps:
    sudo apt-get update
    sudo apt-get install -y libudev-dev libasound2-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libwebkit2gtk-4.1-dev libspeechd-dev glslang-tools

install-deps:
    if (-not (Get-Command mise -ErrorAction SilentlyContinue)) { cargo install mise --locked }
    mise install

install-tools:
    cargo install --locked cargo-machete

lint: lint-wasm lint-crates-isolated lint-apps-isolated lint-features check-feature-forwarding lint-dag lint-examples lint-shaders lint-vulkan-shaders
    cargo clippy --all --all-targets -- -D warnings

lint-dag:
    cargo run --quiet -p dag-check

# Show only the numbers the project file states that the tree does not hold
lint-counts:
    $ErrorActionPreference = 'Stop'; $report = (cargo run --quiet -p dag-check 2>&1) | Out-String; $lines = @($report -split "`n"); $at = [Array]::FindIndex($lines, [Predicate[string]]{ param($line) $line -match 'states a number the tree does not hold' }); if ($at -ge 0) { $lines[$at..($lines.Count - 1)] | Where-Object { $_ -match 'the file says|states no number' } | ForEach-Object { Write-Host $_.TrimEnd() }; throw 'the project file states a number the tree does not hold' }; Write-Host 'every number the project file states is the number the tree holds'

# Show only the layout faults that name one crate, so a new crate is checked while it is still small
check-crate name:
    $ErrorActionPreference = 'Stop'; $report = (cargo run --quiet -p dag-check 2>&1) | Out-String; $hits = @($report -split "`n" | Where-Object { $_ -match [regex]::Escape('{{name}}') }); if ($hits.Count -gt 0) { $hits | ForEach-Object { Write-Host $_.TrimEnd() }; throw '{{name}} does not hold the layout' }; Write-Host '{{name}} holds the layout'

# Write a new plugin crate on the bands of template/, so it cannot start the wrong shape
new-plugin name level:
    $ErrorActionPreference = 'Stop'; & ./tools/new-plugin.ps1 -Name '{{name}}' -Level '{{level}}'

# Fail when an example stands in for the crate it shows
lint-examples:
    cargo run --quiet -p example-check

# Print a digest of a game's generated world, so one build can be compared with another
digest game="voxels" *arguments="":
    cargo run --quiet --release -p world-digest -- {{game}} {{arguments}}

lint-shaders:
    cargo run --quiet -p shader-check

lint-vulkan-shaders:
    $ErrorActionPreference = 'Stop'; $stages = '.vert', '.frag', '.comp', '.mesh', '.task', '.geom', '.tesc', '.tese'; $output = Join-Path ([IO.Path]::GetTempPath()) ([IO.Path]::GetRandomFileName()); New-Item -ItemType Directory -Force $output | Out-Null; try { foreach ($shader in Get-ChildItem -File -Recurse -Path apps, crates | Where-Object { $_.Extension -in $stages }) { glslangValidator -V --target-env vulkan1.3 -o (Join-Path $output "$($shader.Name).spv") $shader.FullName; if ($LASTEXITCODE -ne 0) { throw "the shader did not compile: $($shader.Name)" } } } finally { Remove-Item -Recurse -Force $output } ; Write-Host 'every vulkan shader compiles' 

lint-apps-isolated:
    $ErrorActionPreference = 'Stop'; foreach ($directory in @(Get-ChildItem -Directory -Recurse -Depth 1 apps) + @(Get-Item template)) { if (-not (Test-Path (Join-Path $directory.FullName 'Cargo.toml'))) { continue }; $name = $directory.Name; Write-Host "check (isolated): $name"; cargo check -p $name; if ($LASTEXITCODE -ne 0) { throw "isolated check failed for $name" } }

lint-crates-isolated:
    $ErrorActionPreference = 'Stop'; foreach ($directory in Get-ChildItem -Directory crates, crates/plugins/*/*, crates/framework/*/*) { if (-not (Test-Path (Join-Path $directory.FullName 'Cargo.toml'))) { continue }; $name = $directory.Name; Write-Host "clippy (isolated): $name"; cargo clippy -p $name --lib --bins -- -D warnings; if ($LASTEXITCODE -ne 0) { throw "isolated clippy failed for $name" } }

lint-features:
    $ErrorActionPreference = 'Stop'; foreach ($directory in @(Get-ChildItem -Directory crates, crates/plugins/*/*, crates/framework/*/*) + @(Get-Item template)) { $manifest = Join-Path $directory.FullName 'Cargo.toml'; if (-not (Test-Path $manifest)) { continue }; if ((Get-Content $manifest -Raw) -notmatch '(?m)^\s*contract\s*=') { continue }; $name = $directory.Name; Write-Host "check (contract): $name"; cargo check -p $name --no-default-features --features contract; if ($LASTEXITCODE -ne 0) { throw "$name does not build with only its contract" } }

lint-wasm:
    $ErrorActionPreference = 'Stop'; $excluded = @('dag-check', 'jarvis', 'multiplayer_sandbox', 'nightshade-openxr', 'nightshade-steam', 'nightshade-vulkan'); foreach ($manifest in Get-ChildItem -Path apps -Recurse -Depth 3 -Filter Cargo.toml -File) { $body = Get-Content $manifest.FullName -Raw; if ($body -match '(?m)^\s*default\s*=\s*\[[^\]]*"vulkan"') { $excluded += [regex]::Match($body, '(?m)^name = "([^"]*)"').Groups[1].Value } }; $arguments = @('clippy', '--target', 'wasm32-unknown-unknown', '--workspace'); foreach ($name in $excluded) { $arguments += @('--exclude', $name) }; $arguments += @('--', '-D', 'warnings'); cargo @arguments

check-feature-forwarding:
    $ErrorActionPreference = 'Stop'; $missing = @(); foreach ($directory in Get-ChildItem -Directory crates, crates/plugins/*/*, crates/framework/*/*) { $manifest = Join-Path $directory.FullName 'Cargo.toml'; $source = Join-Path $directory.FullName 'src'; if (-not (Test-Path $manifest) -or -not (Test-Path $source)) { continue }; $text = Get-Content $manifest -Raw; $names = Select-String -Path (Get-ChildItem -Recurse -Filter *.rs $source) -Pattern 'feature = "([a-zA-Z_0-9-]+)"' -AllMatches | ForEach-Object { $_.Matches } | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique; foreach ($name in $names) { if ($text -notmatch "(?m)^\s*$([regex]::Escape($name))\s*=") { $missing += "$($directory.Name) gates on '$name' but declares no such feature" } } }; if ($missing.Count -gt 0) { $missing | ForEach-Object { Write-Host "undeclared feature: $_" }; throw 'feature declaration check failed' }

search *query:
    @$ErrorActionPreference = 'Stop'; $esc = [char]27; $dim = "$esc[2m"; $bold = "$esc[1m"; $off = "$esc[0m"; $tab = [char]9; $tint = @{ games = "$esc[38;5;213m"; features = "$esc[38;5;80m"; ui = "$esc[38;5;222m"; rendering = "$esc[38;5;150m"; multiplayer = "$esc[38;5;209m" }; $rows = Get-ChildItem -Path apps -Recurse -Depth 3 -Filter Cargo.toml -File | ForEach-Object { $body = Get-Content $_.FullName -Raw; $name = [regex]::Match($body, '(?m)^name = "([^"]*)"').Groups[1].Value; $about = [regex]::Match($body, '(?m)^description = "([^"]*)"').Groups[1].Value; [pscustomobject]@{ Group = $_.Directory.Parent.Name; Name = $name; About = $about } } | Sort-Object Group, Name; $width = ($rows | ForEach-Object { $_.Name.Length } | Measure-Object -Maximum).Maximum; $lines = $rows | ForEach-Object { $tag = $tint[$_.Group]; if (-not $tag) { $tag = "" }; $_.Name + $tab + " " + $tag + $_.Group.PadRight(9) + $off + " " + $bold + $_.Name.PadRight($width) + $off + "  " + $dim + $_.About + $off }; $options = @("--ansi", "--delimiter", $tab, "--with-nth", "2", "--height", "80%", "--layout", "reverse", "--border", "rounded", "--border-label", " nightshade apps ", "--prompt", "run > ", "--pointer", "|>", "--info", "inline", "--header", "enter runs the app, esc cancels", "--header-first", "--color", "border:cyan,label:bold:cyan,prompt:green,pointer:magenta,hl:yellow,hl+:yellow,info:dim"); $seed = "{{query}}"; if ($seed) { $options += @("--query", $seed) }; $picked = $lines | fzf @options; if (-not $picked) { exit 0 }; $app = $picked.Split($tab)[0]; Write-Host "$esc[38;5;80mrunning$esc[0m $app"; cargo run -r -p $app

search-vulkan *query:
    @$ErrorActionPreference = 'Stop'; $esc = [char]27; $dim = "$esc[2m"; $bold = "$esc[1m"; $off = "$esc[0m"; $tab = [char]9; $tint = @{ games = "$esc[38;5;213m"; features = "$esc[38;5;80m"; ui = "$esc[38;5;222m"; rendering = "$esc[38;5;150m" }; $rows = Get-ChildItem -Path apps -Recurse -Depth 3 -Filter Cargo.toml -File | ForEach-Object { $body = Get-Content $_.FullName -Raw; if ($body -notmatch '(?m)^\s*vulkan\s*=') { return }; $name = [regex]::Match($body, '(?m)^name = "([^"]*)"').Groups[1].Value; $about = [regex]::Match($body, '(?m)^description = "([^"]*)"').Groups[1].Value; [pscustomobject]@{ Group = $_.Directory.Parent.Name; Name = $name; About = $about } } | Sort-Object Group, Name; if (-not $rows) { throw 'no app declares a vulkan feature' }; $width = ($rows | ForEach-Object { $_.Name.Length } | Measure-Object -Maximum).Maximum; $lines = $rows | ForEach-Object { $tag = $tint[$_.Group]; if (-not $tag) { $tag = "" }; $_.Name + $tab + " " + $tag + $_.Group.PadRight(9) + $off + " " + $bold + $_.Name.PadRight($width) + $off + "  " + $dim + $_.About + $off }; $options = @("--ansi", "--delimiter", $tab, "--with-nth", "2", "--height", "80%", "--layout", "reverse", "--border", "rounded", "--border-label", " nightshade vulkan apps ", "--prompt", "run > ", "--pointer", "|>", "--info", "inline", "--header", "enter runs the app on vulkan, esc cancels", "--header-first", "--color", "border:cyan,label:bold:cyan,prompt:green,pointer:magenta,hl:yellow,hl+:yellow,info:dim"); $seed = "{{query}}"; if ($seed) { $options += @("--query", $seed) }; $picked = $lines | fzf @options; if (-not $picked) { exit 0 }; $app = $picked.Split($tab)[0]; Write-Host "$esc[38;5;80mrunning$esc[0m $app $esc[2mon vulkan$esc[0m"; cargo run -r -p $app --no-default-features --features vulkan

search-wasm *query:
    @$ErrorActionPreference = 'Stop'; $esc = [char]27; $dim = "$esc[2m"; $bold = "$esc[1m"; $off = "$esc[0m"; $tab = [char]9; $tint = @{ games = "$esc[38;5;213m"; features = "$esc[38;5;80m"; ui = "$esc[38;5;222m"; rendering = "$esc[38;5;150m" }; $rows = Get-ChildItem -Path apps -Recurse -Depth 3 -Filter Cargo.toml -File | ForEach-Object { $body = Get-Content $_.FullName -Raw; if (-not (Test-Path (Join-Path $_.Directory.FullName 'Trunk.toml'))) { return }; $name = [regex]::Match($body, '(?m)^name = "([^"]*)"').Groups[1].Value; $about = [regex]::Match($body, '(?m)^description = "([^"]*)"').Groups[1].Value; [pscustomobject]@{ Group = $_.Directory.Parent.Name; Name = $name; About = $about } } | Sort-Object Group, Name; if (-not $rows) { throw 'no app carries a Trunk.toml' }; $width = ($rows | ForEach-Object { $_.Name.Length } | Measure-Object -Maximum).Maximum; $lines = $rows | ForEach-Object { $tag = $tint[$_.Group]; if (-not $tag) { $tag = "" }; $_.Name + $tab + " " + $tag + $_.Group.PadRight(9) + $off + " " + $bold + $_.Name.PadRight($width) + $off + "  " + $dim + $_.About + $off }; $options = @("--ansi", "--delimiter", $tab, "--with-nth", "2", "--height", "80%", "--layout", "reverse", "--border", "rounded", "--border-label", " nightshade web apps ", "--prompt", "run > ", "--pointer", "|>", "--info", "inline", "--header", "enter serves the app in a browser, esc cancels", "--header-first", "--color", "border:cyan,label:bold:cyan,prompt:green,pointer:magenta,hl:yellow,hl+:yellow,info:dim"); $seed = "{{query}}"; if ($seed) { $options += @("--query", $seed) }; $picked = $lines | fzf @options; if (-not $picked) { exit 0 }; $app = $picked.Split($tab)[0]; Write-Host "$esc[38;5;80mrunning$esc[0m $app $esc[2min a browser$esc[0m"; $manifest = (Get-ChildItem -Path apps -Recurse -Depth 3 -Filter Trunk.toml -File | Where-Object { (Get-Content (Join-Path $_.Directory.FullName 'Cargo.toml') -Raw) -match ('(?m)^name = "' + [regex]::Escape($app) + '"') } | Select-Object -First 1); if (-not $manifest) { throw "no trunk configuration for $app" }; trunk serve --release --open --config $manifest.FullName


run project="sandbox" *arguments="":
    cargo run -r -p {{project}} -- {{arguments}}

run-vulkan $project="boids_gpu" *arguments="":
    cargo run -r -p {{project}} --no-default-features --features vulkan -- {{arguments}}

run-web:
    trunk serve --release --open --config apps/features/fireworks/Trunk.toml

run-app $project="sandbox" *arguments="":
    cargo run -r -p {{project}} -- {{arguments}}

run-vr $name="openxr_hello_vr":
    cargo run -r -p nightshade-openxr --example {{name}}

# Run the example that sits beside a framework crate
example name="health":
    @$ErrorActionPreference = 'Stop'; $found = Get-ChildItem -Path crates/framework -Recurse -Depth 4 -Directory | Where-Object { $_.Name -eq '{{name}}' -and $_.Parent.Name -eq 'examples' } | Select-Object -First 1; if (-not $found) { throw 'no framework crate carries an example named {{name}}' }; $crate = $found.Parent.Parent.Name; Write-Host "running $crate example {{name}}"; cargo run -r -p $crate --example {{name}}

# Pick a framework example from a list of every one
search-examples *query:
    @$ErrorActionPreference = 'Stop'; $esc = [char]27; $dim = "$esc[2m"; $bold = "$esc[1m"; $off = "$esc[0m"; $tab = [char]9; $tint = @{ base = "$esc[38;5;80m"; world = "$esc[38;5;150m"; state = "$esc[38;5;222m"; mechanics = "$esc[38;5;213m"; interface = "$esc[38;5;209m"; tools = "$esc[38;5;110m" }; $rows = Get-ChildItem -Path crates/framework -Recurse -Depth 4 -Directory | Where-Object { $_.Parent.Name -eq 'examples' } | ForEach-Object { $crate = $_.Parent.Parent; $group = $crate.Parent.Name; $about = [regex]::Match((Get-Content (Join-Path $crate.FullName 'Cargo.toml') -Raw), '(?m)^description = "([^"]*)"').Groups[1].Value; [pscustomobject]@{ Group = $group; Crate = $crate.Name; Name = $_.Name; About = $about } } | Sort-Object Group, Name; if (-not $rows) { throw 'no framework crate carries an example' }; $width = ($rows | ForEach-Object { $_.Name.Length } | Measure-Object -Maximum).Maximum; $lines = $rows | ForEach-Object { $tag = $tint[$_.Group]; if (-not $tag) { $tag = "" }; $_.Crate + $tab + $_.Name + $tab + " " + $tag + $_.Group.PadRight(9) + $off + " " + $bold + $_.Name.PadRight($width) + $off + "  " + $dim + $_.About + $off }; $options = @("--ansi", "--delimiter", $tab, "--with-nth", "3", "--height", "80%", "--layout", "reverse", "--border", "rounded", "--border-label", " nightshade framework examples ", "--prompt", "run > ", "--pointer", "|>", "--info", "inline", "--header", "enter runs the example, esc cancels", "--header-first", "--color", "border:cyan,label:bold:cyan,prompt:green,pointer:magenta,hl:yellow,hl+:yellow,info:dim"); $seed = "{{query}}"; if ($seed) { $options += @("--query", $seed) }; $picked = $lines | fzf @options; if (-not $picked) { exit 0 }; $parts = $picked.Split($tab); Write-Host "$esc[38;5;80mrunning$esc[0m $($parts[0]) example $($parts[1])"; cargo run -r -p $parts[0] --example $parts[1]

# Serve the example that sits beside a framework crate in a browser
example-web name="health":
    @$ErrorActionPreference = 'Stop'; $found = Get-ChildItem -Path crates/framework -Recurse -Depth 4 -Directory | Where-Object { $_.Name -eq '{{name}}' -and $_.Parent.Name -eq 'examples' } | Select-Object -First 1; if (-not $found) { throw 'no framework crate carries an example named {{name}}' }; $crate = $found.Parent.Parent; $manifest = Join-Path $crate.FullName 'Trunk.toml'; if (-not (Test-Path $manifest)) { throw "$($crate.Name) carries no Trunk.toml" }; Write-Host "serving $($crate.Name) example {{name}}"; trunk serve --release --open --example {{name}} --config $manifest

# Pick a framework example from a list of every one and serve it in a browser
search-examples-web *query:
    @$ErrorActionPreference = 'Stop'; $esc = [char]27; $dim = "$esc[2m"; $bold = "$esc[1m"; $off = "$esc[0m"; $tab = [char]9; $tint = @{ base = "$esc[38;5;80m"; world = "$esc[38;5;150m"; state = "$esc[38;5;222m"; mechanics = "$esc[38;5;213m"; interface = "$esc[38;5;209m"; tools = "$esc[38;5;110m" }; $rows = Get-ChildItem -Path crates/framework -Recurse -Depth 4 -Directory | Where-Object { $_.Parent.Name -eq 'examples' } | ForEach-Object { $crate = $_.Parent.Parent; if (-not (Test-Path (Join-Path $crate.FullName 'Trunk.toml'))) { return }; $group = $crate.Parent.Name; $about = [regex]::Match((Get-Content (Join-Path $crate.FullName 'Cargo.toml') -Raw), '(?m)^description = "([^"]*)"').Groups[1].Value; [pscustomobject]@{ Group = $group; Crate = $crate.FullName; Name = $_.Name; About = $about } } | Sort-Object Group, Name; if (-not $rows) { throw 'no framework crate carries an example with a Trunk.toml' }; $width = ($rows | ForEach-Object { $_.Name.Length } | Measure-Object -Maximum).Maximum; $lines = $rows | ForEach-Object { $tag = $tint[$_.Group]; if (-not $tag) { $tag = "" }; $_.Crate + $tab + $_.Name + $tab + " " + $tag + $_.Group.PadRight(9) + $off + " " + $bold + $_.Name.PadRight($width) + $off + "  " + $dim + $_.About + $off }; $options = @("--ansi", "--delimiter", $tab, "--with-nth", "3", "--height", "80%", "--layout", "reverse", "--border", "rounded", "--border-label", " nightshade framework examples on the web ", "--prompt", "serve > ", "--pointer", "|>", "--info", "inline", "--header", "enter serves the example in a browser, esc cancels", "--header-first", "--color", "border:cyan,label:bold:cyan,prompt:green,pointer:magenta,hl:yellow,hl+:yellow,info:dim"); $seed = "{{query}}"; if ($seed) { $options += @("--query", $seed) }; $picked = $lines | fzf @options; if (-not $picked) { exit 0 }; $parts = $picked.Split($tab); Write-Host "$esc[38;5;80mserving$esc[0m $($parts[1]) $esc[2min a browser$esc[0m"; trunk serve --release --open --example $parts[1] --config (Join-Path $parts[0] 'Trunk.toml')

run-wasm $project="fireworks":
    $ErrorActionPreference = 'Stop'; $config = Get-ChildItem -Path apps -Recurse -Filter Trunk.toml | Where-Object { $_.Directory.Name -eq $env:project } | Select-Object -First 1; if (-not $config) { throw "no app named $env:project under apps" }; trunk serve --release --open --config $config.FullName

# A browser refuses an ES module and a wasm fetch from a file:// address, so
# opening index.html from the disk shows a blank page. The site plays from any
# HTTP address, GitHub Pages included, and `just serve-export` gives it one
# here. `--public-url ./` writes relative paths, so the site plays at the root
# of a domain or in a subdirectory. Trunk clears the directory it writes to.
#
# What is in `export/` lands on top of the build, which makes the directory a
# repository you can push: a README and a Pages workflow that uploads these
# files and deploys them. The workflow compiles nothing, because the wasm is
# already there.
#
# Builds an app for the web as a static site you can host or send
export-wasm $project="fireworks" $directory="":
    $ErrorActionPreference = 'Stop'; $config = Get-ChildItem -Path apps -Recurse -Filter Trunk.toml -File | Where-Object { $_.Directory.Name -eq $env:project } | Select-Object -First 1; if (-not $config) { throw "no app named $env:project under apps" }; $out = if ($env:directory) { $env:directory } else { "web/$env:project" }; if (-not [IO.Path]::IsPathRooted($out)) { $out = Join-Path (Get-Location) $out }; $out = [IO.Path]::GetFullPath($out); trunk build --release --config $config.FullName --dist $out --public-url ./; if ($LASTEXITCODE -ne 0) { throw "the web build failed for $env:project" }; New-Item -ItemType File -Force (Join-Path $out '.nojekyll') | Out-Null; Get-ChildItem -Force (Join-Path (Get-Location) 'export') | Copy-Item -Recurse -Force -Destination $out; Write-Host "the site is at $out"

# The whitelist is the `export_apps` variable at the top of this file. Each app
# lands in web/<name>/ and the landing page at web/index.html links to them,
# which is the whole tree GitHub Pages wants. This recipe owns web/ and clears
# it first, so the page lists what is there and nothing else.
#
# GitHub reads a workflow only from the root of a repository, so the README and
# the Pages workflow land at web/ and are taken back out of each app folder.
#
# Builds every app on the whitelist into one static site
export-wasm-all:
    $ErrorActionPreference = 'Stop'; $names = '{{export_apps}}'.Split(' ', [StringSplitOptions]::RemoveEmptyEntries); $root = [IO.Path]::GetFullPath((Join-Path (Get-Location) 'web')); if (Test-Path $root) { Remove-Item -Recurse -Force $root }; New-Item -ItemType Directory -Force $root | Out-Null; $cards = @(); foreach ($name in $names) { just export-wasm $name; if ($LASTEXITCODE -ne 0) { throw "the export failed for $name" }; $manifest = Get-ChildItem -Path apps -Recurse -Depth 2 -Filter Cargo.toml -File | Where-Object { (Get-Content $_.FullName -Raw) -match ('(?m)^name = "' + [regex]::Escape($name) + '"') } | Select-Object -First 1; $about = if ($manifest) { [regex]::Match((Get-Content $manifest.FullName -Raw), '(?m)^description = "([^"]*)"').Groups[1].Value } else { '' }; $cards += '<a class="card" href="' + $name + '/"><h2>' + $name + '</h2><p>' + $about + '</p></a>' }; $style = 'html { color-scheme: dark } body { margin: 0; padding: 4rem 2rem; background: #0d0c11; color: #e8e6ef; font: 16px/1.6 system-ui, sans-serif } h1 { margin: 0 0 2.5rem; font-size: 2rem; letter-spacing: -0.02em } .grid { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fill, minmax(19rem, 1fr)); max-width: 62rem } .card { display: block; padding: 1.25rem 1.5rem; border: 1px solid #262433; border-radius: 12px; background: #15131c; color: inherit; text-decoration: none } .card:hover { border-color: #7c5cff; background: #191627 } .card h2 { margin: 0 0 0.4rem; font-size: 1.1rem } .card p { margin: 0; color: #9a95ad; font-size: 0.9rem }'; $page = @('<!doctype html>', '<html lang="en">', '<head>', '<meta charset="utf-8" />', '<meta name="viewport" content="width=device-width, initial-scale=1" />', '<title>nightshade</title>', '<style>' + $style + '</style>', '</head>', '<body>', '<h1>nightshade</h1>', '<div class="grid">') + $cards + @('</div>', '</body>', '</html>'); [IO.File]::WriteAllText((Join-Path $root 'index.html'), ($page -join "`n")); New-Item -ItemType File -Force (Join-Path $root '.nojekyll') | Out-Null; foreach ($name in $names) { foreach ($extra in @('.github', 'README.md')) { $path = Join-Path $root (Join-Path $name $extra); if (Test-Path $path) { Remove-Item -Recurse -Force $path } } }; Get-ChildItem -Force (Join-Path (Get-Location) 'export') | Copy-Item -Recurse -Force -Destination $root; Write-Host "the site is at $root"

# The media type of a .wasm file must be application/wasm, or
# WebAssembly.instantiateStreaming refuses it, which is why this is a listener
# with its own table and not a one-line file share.
#
# Serves an exported site on localhost so a browser will play it
serve-export $directory="web" $port="8080":
    $ErrorActionPreference = 'Stop'; $root = $env:directory; if (-not [IO.Path]::IsPathRooted($root)) { $root = Join-Path (Get-Location) $root }; $root = [IO.Path]::GetFullPath($root); if (-not (Test-Path $root)) { throw "no directory at $root, run 'just export-wasm' first" }; $types = @{ '.html' = 'text/html'; '.js' = 'text/javascript'; '.mjs' = 'text/javascript'; '.wasm' = 'application/wasm'; '.css' = 'text/css'; '.json' = 'application/json'; '.png' = 'image/png'; '.jpg' = 'image/jpeg'; '.svg' = 'image/svg+xml'; '.ico' = 'image/x-icon'; '.ogg' = 'audio/ogg'; '.wav' = 'audio/wav'; '.glb' = 'model/gltf-binary'; '.hdr' = 'image/vnd.radiance'; '.ttf' = 'font/ttf' }; $listener = New-Object Net.HttpListener; $address = "http://127.0.0.1:$env:port/"; $listener.Prefixes.Add($address); $listener.Start(); Write-Host "serving $root at $address"; Start-Process $address; try { while ($listener.IsListening) { $context = $listener.GetContext(); $path = [Uri]::UnescapeDataString($context.Request.Url.AbsolutePath).TrimStart('/'); if (-not $path) { $path = 'index.html' }; $file = [IO.Path]::GetFullPath((Join-Path $root $path)); if (Test-Path $file -PathType Container) { $file = Join-Path $file 'index.html' }; if (-not $file.StartsWith($root)) { $context.Response.StatusCode = 403 } elseif (Test-Path $file -PathType Leaf) { $bytes = [IO.File]::ReadAllBytes($file); $type = $types[[IO.Path]::GetExtension($file).ToLower()]; if (-not $type) { $type = 'application/octet-stream' }; $context.Response.ContentType = $type; $context.Response.OutputStream.Write($bytes, 0, $bytes.Length) } else { $context.Response.StatusCode = 404 }; $context.Response.Close() } } finally { $listener.Stop() }

serve-book:
    trunk serve --release --open --config apps/book/Trunk.toml --dist (Join-Path (Get-Location) 'book')

test:
    cargo test --all -- --nocapture

udeps:
    cargo machete

vendor:
    cargo vendor

@versions:
    rustc --version
    cargo fmt -- --version
    cargo clippy -- --version

watch $project="sandbox":
    cargo watch -x 'run -r -p {{project}}'

sync-toolchain:
    $ErrorActionPreference = 'Stop'; $channel = (Select-String -Path 'rust-toolchain' -Pattern '^channel = "(.+)"').Matches.Groups[1].Value; Write-Host "Pinning the templates to Rust $channel"; foreach ($t in 'template') { $path = Join-Path (Get-Location) "$t/rust-toolchain"; $body = (Get-Content $path -Raw) -replace '(?m)^channel = ".*"', "channel = `"$channel`""; [IO.File]::WriteAllText($path, $body.Replace("`r`n", "`n")) }

build-steamdeck $project="sandbox":
    cross build --release --target x86_64-unknown-linux-gnu -p {{project}}

deploy-steamdeck $project="sandbox":
    just build-steamdeck {{project}}
    scp ./target/x86_64-unknown-linux-gnu/release/{{project}} deck@steamdeck.local:~/Downloads

deploy-steamdeck-quick $project="sandbox":
    just build-steamdeck {{project}}
    scp ./target/x86_64-unknown-linux-gnu/release/{{project}} deck@steamdeck.local:~/Downloads/game
    ssh deck@steamdeck.local "chmod +x ~/Downloads/game"

steamdeck-ssh:
    ssh deck@steamdeck.local

# The packager brings its own ndk, so the machine needs only a jdk, clang, and
# the platform tools, which `x doctor` reports on.
#
# Installs the Android toolchain
init-android:
    rustup target add aarch64-linux-android
    cargo install --git https://github.com/rust-mobile/xbuild --bin x --locked
    x doctor

# basis-universal and anything else in the graph that compiles C or C++ goes
# through the `cc` crate, and a host clang carries no android headers, so it
# needs a sysroot. ANDROID_NDK_SYSROOT wins if set, then a full
# ANDROID_NDK_ROOT install, then the one the packager bundles. Missing entirely
# is reported here rather than as a header cc cannot find.
[private]
android-sysroot:
    @$ErrorActionPreference = 'Stop'; $s = if ($env:ANDROID_NDK_SYSROOT) { $env:ANDROID_NDK_SYSROOT } elseif ($env:ANDROID_NDK_ROOT) { Join-Path $env:ANDROID_NDK_ROOT 'toolchains\llvm\prebuilt\windows-x86_64\sysroot' } else { Join-Path $env:LOCALAPPDATA 'x\Android.ndk' }; if (-not (Test-Path $s)) { throw "no android sysroot at $s, run 'just init-android'" }; Write-Output $s

# An app carries an X.toml and a manifest.yaml beside its Cargo.toml, builds a
# cdylib as well as a binary, and declares its entry point with
# `nightshade_platform::android_main!`. The template shows all three.
#
# Builds an app as an Android apk into target/x/release/android/
build-android $project="template":
    $ErrorActionPreference = 'Stop'; $s = just android-sysroot; $env:CC_aarch64_linux_android = 'clang'; $env:CXX_aarch64_linux_android = 'clang++'; $env:AR_aarch64_linux_android = 'llvm-ar'; $env:CFLAGS_aarch64_linux_android = "--sysroot=$s"; $env:CXXFLAGS_aarch64_linux_android = "--sysroot=$s"; $crate = @(Get-ChildItem -Path apps -Recurse -Depth 3 -Filter Cargo.toml -File) + @(Get-Item template/Cargo.toml) | Where-Object { (Get-Content $_.FullName -Raw) -match ('(?m)^name = "' + [regex]::Escape($env:project) + '"') } | Select-Object -First 1; if (-not $crate) { throw "no crate named $env:project under apps or template" }; foreach ($file in 'X.toml', 'manifest.yaml') { if (-not (Test-Path (Join-Path $crate.Directory.FullName $file))) { throw "$env:project has no $file, so the packager has nothing to describe an apk with" } }; x build --release --platform android --arch arm64 --manifest-path $crate.FullName

# Installs an already built apk on a connected device
install-android $project="template":
    adb install -r target/x/release/android/{{project}}.apk

# Builds, installs, and starts an app on a connected device
run-android $project="template":
    just build-android {{project}}
    just install-android {{project}}
    adb shell monkey -p com.nightshade.{{project}} -c android.intent.category.LAUNCHER 1

# The tag is the window title the app composed, which is what reaches logcat.
#
# Follows the running app's log
logcat-android $tag="Template":
    adb logcat -s {{tag}}

# A phone on wireless debugging shows under the services until it is connected
# and under the devices after. Nothing under the services means the phone is on
# a different network, or its Wireless debugging screen is closed, or the
# network carries no mdns. Then read the address off the phone and give it to
# `just connect-android`.
#
# Lists connected Android devices, and the wireless ones this machine can see
list-android:
    adb devices -l
    adb mdns services

# The phone's Wireless debugging screen has a "Pair device with pairing code"
# button, which shows an address and a six digit code. Both go here. A phone
# stays paired, so this is one time for each machine, and a phone that is
# already paired needs `just connect-android` alone.
#
# Pairs with a phone over wireless debugging
pair-android $address $code:
    adb pair {{address}} {{code}}

# The address at the top of the phone's Wireless debugging screen, which is on
# a different port from the pairing one and changes whenever the phone turns
# wireless debugging off and on again.
#
# Connects to a paired phone over wireless debugging
connect-android $address:
    adb connect {{address}}

# The two host tools compile for the machine that runs them, openxr and steam
# link libraries built for the host, and the vulkan backend has no android
# surface.
#
# Runs clippy for the Android target
lint-android:
    $ErrorActionPreference = 'Stop'; $s = just android-sysroot; $env:CC_aarch64_linux_android = 'clang'; $env:CXX_aarch64_linux_android = 'clang++'; $env:AR_aarch64_linux_android = 'llvm-ar'; $env:CFLAGS_aarch64_linux_android = "--sysroot=$s"; $env:CXXFLAGS_aarch64_linux_android = "--sysroot=$s"; $excluded = @('dag-check', 'shader-check', 'jarvis', 'nightshade-openxr', 'nightshade-steam', 'nightshade-vulkan'); foreach ($manifest in Get-ChildItem -Path apps -Recurse -Depth 3 -Filter Cargo.toml -File) { $body = Get-Content $manifest.FullName -Raw; if ($body -match '(?m)^\s*default\s*=\s*\[[^\]]*"vulkan"') { $excluded += [regex]::Match($body, '(?m)^name = "([^"]*)"').Groups[1].Value } }; $arguments = @('clippy', '--target', 'aarch64-linux-android', '--workspace'); foreach ($name in $excluded) { $arguments += @('--exclude', $name) }; $arguments += @('--', '-D', 'warnings'); cargo @arguments

# Compile every GLSL shader in the workspace to SPIR-V
compile-vulkan-shaders:
    $ErrorActionPreference = 'Stop'; $stages = '.vert', '.frag', '.comp', '.mesh', '.task', '.geom', '.tesc', '.tese'; foreach ($shader in Get-ChildItem -File -Recurse -Path apps, crates | Where-Object { $_.Extension -in $stages }) { Write-Host "compiling: $($shader.Name)"; glslangValidator -V --target-env vulkan1.3 -o "$($shader.FullName).spv" $shader.FullName; if ($LASTEXITCODE -ne 0) { throw "the shader did not compile: $($shader.Name)" } }
