# DisplaySwitch Technical Notes

**English** · [简体中文](TECHNICAL.md)

One line on the stack: **Tauri 2 (Rust backend) + Vue 3 (Options API) + Vite frontend, with the actual switching delegated to the bundled [displayplacer](https://github.com/jakehilborn/displayplacer)**. Tauri over Electron means a small binary and the system WebView; and rather than driving CoreGraphics by hand, we read the state, assign roles, and generate a spec for a CLI that already does this well.

Everything user-facing lives in the [README](../README.en.md). This file is only about how it works and why it works that way.

## Architecture and the tray window

- **Window**: label `main`, 600×520 logical points, `decorations:false` + `transparent:true` + `alwaysOnTop` + `skipTaskbar` + `visible:false` (`src-tauri/tauri.conf.json`). With a single display attached the width narrows to 380 (`SINGLE_PANEL_WIDTH`) — the width is chosen at tray-click time from `LAST_DISPLAY_COUNT`, which every `get_state` refreshes, so plugging displays in or out does not require restarting the app to get the right width.
- **Look**: `window-vibrancy`'s `apply_vibrancy(NSVisualEffectMaterial::HudWindow)` — combined with the rounded cards, that is the macOS HUD frosted-glass material.
- **No Dock presence**: `LSUIElement = true` in `src-tauri/Info.plist` makes LaunchServices register the process as a background agent at check-in, so there is no Dock icon and no ⌘Tab entry — just the menu bar glyph. The `set_activation_policy(Accessory)` call in `setup` stays as the fallback for the bare `cargo run` binary, which never reads an Info.plist (in that mode the Dock icon still flashes once at startup — normal).
- **Hides on blur**: `WindowEvent::Focused(false)` → `hide_panel_window()`. Clicking anywhere outside the panel, or switching apps, dismisses it; the × in the header calls the `hide_panel` command for the same effect. The one exception is quick switch, where `PANEL_LOCK` holds it open — see "External launch: quick switch".
- **Tray**: `TrayIconBuilder::with_id("main-tray")` with `icon_as_template(true)`. Left click toggles the panel, right click opens the native menu (Show Panel / Settings… / Quit). A native menu does not re-render with the webview, so changing language rebuilds it in place via `tray_by_id("main-tray").set_menu(Some(tray_menu(app)))`.
- **Positioning**: the click coordinates reported by the tray are physical pixels, so they are divided by `window.scale_factor()` before positioning; window size itself is expressed in logical points. Mixing the two is what makes Retina 2x placement go wrong.
- **Hotplug**: `CGDisplayRegisterReconfigurationCallback` registered in `setup`. Lid close/open and plug/unplug fire the callback several times in a row, so an incrementing sequence number gives an 800ms debounce and only one `emit("displays-changed")`; the frontend `listen`s to it and calls `load()`, which also runs whenever the panel regains focus.

## Tauri command reference

`src/api.js` is the only exit point; in the browser the same functions hit the mock (see "Two frontend paths").

| Command | Arguments | Returns | Purpose |
|---|---|---|---|
| `get_state` | — | `AppState` | Current displays, roles, mirror groups, available modes; also refreshes `LAST_DISPLAY_COUNT` |
| `apply_config` | `config` | — | Turns the role draft into displayplacer specs and runs it |
| `save_preset` | `name`, `config` | — | Stores a whole configuration |
| `list_presets` | — | `PresetEntry[]` | Lists every setup, including ones whose displays are unplugged |
| `delete_preset` | `name` | — | Deletes one |
| `get_settings` | — | `AppSettings` | Favorites, pin toggle, language preference, setup order, presets-first toggle |
| `toggle_favorite` | `displayId`, `res` | `AppSettings` | Stars/unstars one mode on one display |
| `set_pin_favorites` | `pin` | `AppSettings` | Whether favorites float to the top |
| `set_lang` | `lang` | `AppSettings` | Persists the preference and rebuilds the tray menu |
| `hide_panel` | — | — | Hides the panel |
| `quit_app` | — | — | Quits (`app.exit(0)`) |
| `get_autostart` | — | `bool` | Reads the real LaunchAgent registration state |
| `set_autostart` | `enabled` | `bool` | Enables/disables login item, returns final state |
| `open_url` | `url` | — | Release pages only — see "Checking for updates" |
| `set_preset_order` | `order` | `AppSettings` | Overwrites the setup order after a drag; an empty array means "never dragged, use the on-disk order" |
| `set_preset_first` | `on` | `AppSettings` | Saved-presets-first toggle |
| `quick_switch_launched` | — | `bool` | Consumes the launch-argument flag when the frontend mounts (read once, then cleared); when true the backend also sizes the panel by display count and opens it centered |
| `set_panel_lock` | `locked` | — | Blocks/allows the hide-on-blur behaviour, used during the quick-switch countdown |

The six settings commands (`get_settings` / `toggle_favorite` / `set_pin_favorites` / `set_lang` / `set_preset_order` / `set_preset_first`) **always return the full, current settings object**; the frontend never guesses. That removes a whole class of "the UI thinks it changed but the disk did not".

App-defined commands need no capability entry — `src-tauri/capabilities/default.json` only carries `core:default`. Plugin commands do, which is why autostart is wrapped in two of our own commands instead.

## Display identity, model name and physical size

- **Identity**: presets and applied configs key on the Serial screen id (`sXXXX`). It survives replugging and port changes; CGDirectDisplayID changes on every re-enumeration and must never be used as a persistent key.
- **One probe**: `src-tauri/src/sysinfo.rs` collects model names and physical sizes in one pass and caches them per ctx-id combination, so counting displays does not spawn `system_profiler` repeatedly.
- **Model name**: `_name` from `system_profiler SPDisplaysDataType -json`, matched to the Contextual id (= CGDirectDisplayID) through `_spdisplays_displayID`.
- **Physical size**: IOKit `IOMobileFramebufferShim` → `DisplayAttributes` → `MaxHorizontal/VerticalImageSize`, converted to diagonal inches, matched to displayplacer's serial via the SerialNumber in the same structure.
- **Why not `CGDisplayScreenSize`**: in a mirrored state it returns the size of the whole mirror group, so a 27″ and a 14″ panel would both report the same number. It is only used as a fallback for the built-in display when EDID is unavailable and that display is not mirroring.

## Reading state: parsing displayplacer output

`src-tauri/src/state.rs` parses the text output of `displayplacer list`; the unit-test fixture is real machine output.

- **Main display**: read from the `- main display` marker on the `Origin` line.
- **Mirror groups**: displayplacer appends one suggested command that reproduces the current state, and mirrored displays collapse into a single `id:main+slave1+slave2` token. Splitting on that token recovers multiple groups, which is far more reliable than comparing resolutions display by display.
- The parser only reads. Every write path regenerates the whole spec; nothing patches state incrementally.

## Applying a configuration: building the spec

`build_specs()` translates the frontend role draft into displayplacer arguments:

- Mirror groups become one `id:target+slave1+slave2 res:... scaling:on`, taking the target display's resolution.
- The main display gets `origin:(0,0)` — displayplacer uses origin to decide which display is main, so "who is main" has to be expressed in coordinates.
- Remaining groups are laid out left to right by width so nothing overlaps.
- Standalone displays carry `hz` / `color_depth`; **mirror groups omit them**, because two mirrored panels do not necessarily agree on refresh rate and including it makes the whole command fail.

The frontend enforces the role rules before submitting (main is unique and transfers automatically on demotion, a mirror target can only be the main or an extended display, a mirrored display follows its target's resolution), so the backend always receives a legal combination.

## Filtering resolution modes

We imitate System Settings so the list never offers a mode that would look wrong: keep only modes whose aspect ratio matches the panel's native one (±2%) and that are either HiDPI (`scaling:on`) or the native resolution; prefer the HiDPI variant when both exist; sort by width descending.

## Storage

- **Setups**: `~/Library/Application Support/com.displayswitch.app/presets/<name>.json`, in the v2 role format. A v1 file (`mode` + `primary`) is converted on read, so existing setups do not vanish.
- **Settings**: `~/Library/Application Support/com.displayswitch.app/settings.json` (`src-tauri/src/settings.rs`):

```json
{ "favorites": { "s1001": ["1512x982"] }, "pinFavorites": true, "lang": "auto" }
```

A missing `pinFavorites` reads as `true` and a missing `lang` as `"auto"`, so old files keep working without a migration step. Favorites are recorded per display (serial id + resolution) and the pinned group keeps the original order inside it.

## Internationalization

No vue-i18n. `src/i18n.js` is one zh/en table plus a `reactive({ pref, lang })`:

- `pref` is the three-state preference the user picked (`auto` / `zh` / `en`) and drives which segment is highlighted; `lang` is the effective language (`auto` resolves through `navigator.language`, i.e. the macOS setting). `setLangPref()` updates both and writes `<html lang>`.
- `main.js` exposes `t()` as the global `$t`. Because templates read `i18n.lang`, switching language is a single re-render — no provide/inject, no mixin.
- Anything with a variable uses a `{name}` placeholder and a full-sentence translation (word order differs; concatenation is not on the table). `shortName()` strips the generic suffix per language (Built-in Display → Built-in), while model names such as U2790B stay untouched.
- Generic display names (Built-in Display) and the Native badge are produced in `api.js normalizeState`, so **changing language must re-run `load()`** for names to update; `load()` therefore reads `getSettings` before `getState`.

On the Rust side there is only `src-tauri/src/lang.rs`: an `AtomicBool` caching the effective language plus `tr(zh, en)` / `tr_fill(zh, en, args)`. Threading a lang parameter through every function would be noise. `resolve()` expands `"auto"` (or a missing key in an old settings file) using the first entry of `defaults read -globalDomain AppleLanguages`; `settings::effective_lang()` is the on-disk entry point. `set_lang` runs **persist → `set_active` with the effective value → rebuild tray menu**; inverting the first two would treat `"auto"` as a literal language.

Every backend error string that can reach the user is bilingual. Note that `format!("...{x}")` inline captured interpolation cannot be used for runtime strings (the bilingual template is a variable), so those go through `tr_fill`'s `{x}` + `.replace()`.

## Checking for updates

`src/update.js`, comparison done entirely in the frontend. **No Tauri updater plugin**, no automatic detection, no background polling — it only touches the network when the user clicks "Check for updates" in About.

- Each source is queried once via `releases/latest`: GitHub `WitsonX/DisplaySwitch`, Gitee `Witson/DisplaySwitch` (public, no token, both send CORS headers). A Chinese UI tries Gitee first, an English UI tries GitHub first, and an unreachable source falls through to the other. Each request has an 8s timeout (`AbortController`).
- The tag is validated against a version-only regex, then compared with `version` from `package.json` (a `v` prefix and suffixes like `-beta` do not participate). Both 404 → "No releases yet"; both erroring → "Check failed". All four outcomes render in that one row; nothing pops up.
- **The release URL is assembled from our own constants plus the tag; the API's `html_url` is never used.** Since the tag already passed the whitelist regex, the resulting URL can only be a release page of our own repositories.
- Opening it goes through the Rust `open_url` command (`Command::new("open")`), which tightens the check once more: only `https://github.com/WitsonX/DisplaySwitch/` and `https://gitee.com/Witson/DisplaySwitch/` prefixes are allowed, anything else returns a bilingual error. `is_release_page()` has a unit test covering prefix confusion and an http downgrade.
- Once a newer version is found the button turns into "View release" and no longer re-checks. To check again, close and reopen the panel — `v-if` destroys the settings view and the state returns to "Not checked".

## Autostart

`tauri-plugin-autostart` (`MacosLauncher::LaunchAgent`, `auto-launch` underneath) is registered as a plugin and wrapped by our own `get_autostart` / `set_autostart` commands, so the frontend never imports `@tauri-apps/plugin-autostart` and no extra capability is needed. The switch is **off by default**; enabling it writes `~/Library/LaunchAgents/DisplaySwitch.plist`.

`is_enabled` simply checks whether the plist exists, so settings.json does not store a second copy of the state — two sources of truth drift, and if the user deletes the plist by hand the switch has to say "off". It is re-read every time the settings view opens.

> During development, do not flip this switch: `npm run tauri dev` runs a debug build, so the login item would register `target/debug/` and keep launching the debug binary at boot.

## External launch: quick switch

A command-line entry for launchers and sticky notes such as MarkLock — start it with the flag and you get "switch, then be gone":

```bash
open -a DisplaySwitch --args -quick-switch   # short alias: -qs
```

- **Only two tokens count**: `QUICK_SWITCH_ARGS = ["-quick-switch", "-qs"]` (`src-tauri/src/lib.rs:33`). Before the builder exists, `run()` scans `std::env::args_os()` and sets the `QUICK_SWITCH` `AtomicBool` if any argument matches. The identifier is deliberately ASCII: a Chinese flag would have to survive AppleScript, clipboard and launcher encodings for no benefit.
- **Consumed once**: the frontend calls `quick_switch_launched` on mount, and `QUICK_SWITCH.swap(false)` reads and clears it in one step. When it returns true the backend runs `show_quick_switch` — width by display count (380 for one, 600 for several), centered on the primary monitor (tray popups have a tray coordinate to anchor to, this path has none, so center is the easiest place to find it), then `show` + `set_focus`. Only that one cold start enters quick switch; every later wake-up is a normal panel.
- **Launching with the flag while it already lives in the menu bar is just a wake-up**: `tauri-plugin-single-instance` hands the argv to the first instance and lets the second process `exit(0)`; the callback only calls `show_panel` when the flag is present, so no quick-switch mode and no countdown. On macOS the plugin detects the running instance through `/tmp/com_displayswitch_app_si.sock` (dots of the bundle id replaced with underscores), which is why the bare dev binary and the copy in `/Applications` recognize each other — whichever starts second quits immediately.
- **The countdown only starts after an apply**: `pushDraft` / `applyPreset` call `startCountdown()` once the successful `load()` finishes; three seconds later `quit_app` runs. Clicking another setup does not restart the countdown (it returns early while `countdown > 0`), and a failed apply shows the error without starting it.
- **The panel may not escape during the countdown**: reconfiguring displays makes the panel lose focus for a moment, and plain hide-on-blur would take the countdown and the cancel buttons away with it. So `set_panel_lock(true)` is called before applying, and released by `stopCountdown()` (countdown finished, "Hide to Menu Bar" pressed, or apply failed). The footer only renders for `quickSwitch && view === 'main'`: while counting it is two buttons — "Hide to Menu Bar" tucks the panel away and leaves the app running, "Quit in {n}s" carries the number on the button that actually quits — and otherwise it is a single line of hint text. The toast sits 58px up so it never covers the countdown.

## Icons

- **The menu bar icon** is a monochrome template glyph (`src-tauri/icons/tray-template.png`, 36×36 = 18pt @2x) with `icon_as_template(true)`, handed to `NSImage.isTemplate` so it renders white on a dark menu bar and dark on a light one. Using the colour app icon directly got it downscaled to 18pt and turned into a smudge on a dark menu bar. The shape is "two displays stacked diagonally, with a stand bar on the front one" — without that stand bar it is indistinguishable from the system's duplicate-screen icon.
- **The app icon** uses the same glyph (white shape on graphite). The "Displays" header icon is an inline SVG of the same geometry (18 viewBox + a mask for the seam), and the 48pt icon in About reads `src/assets/app-icon.png`.
- All three come from one path description in `tools/gen_icons.swift`: run `swift tools/gen_icons.swift tools/out` from the repository root to produce the glyph, the 1024 source, `src/assets/app-icon.png` and light/dark previews; then `npx tauri icon tools/out/app-icon-1024.png` regenerates the icns/ico and every size.
- `src/assets/app-icon.png` is **centre-cropped from the 1024 canvas**, keeping only the rounded square: with its transparent margin intact, the image reads as a white frame when placed on a white card. The 11px `border-radius` in `.a-icon` matches the shape's own corner radius after cropping (185.5/824×48 ≈ 10.8px), so the shadow follows the corner and nothing shows through.
- `tauri icon` leaves `tray-template.png` alone, but remember to copy a freshly generated glyph into `src-tauri/icons/` yourself.

## Naming and bundle identifiers

Chinese name 快切屏, English name DisplaySwitch, Chinese first in the UI. Technical identifiers stay put — bundle file name, `CFBundleName`, bundle id and the LaunchAgent plist remain `DisplaySwitch` / `com.displayswitch.app`; only `src-tauri/Info.plist` merges `CFBundleDisplayName = 快切屏` and `LSUIElement` into the generated Info.plist (Tauri 2 reads and merges it automatically). macOS surfaces such as Login Items then show the Chinese name, while dmg names, scripts and paths never contain non-ASCII characters.

## Two frontend paths

The runtime environment is detected through `window.__TAURI_INTERNALS__`: in a browser, `src/api.js` serves a built-in mock (three fake displays; applying a change really mutates the mock state, so mirroring, pinning, presets and both languages are all clickable) with exactly the same data shape as the real backend. Reviewers can therefore look at the UI without installing Rust.

## Development

Requirements: macOS (macOS only) + Xcode Command Line Tools + Node.js 18+ + a Rust toolchain.

```bash
npm install
npm run tauri dev          # development run (hot reload)
npm run tauri build        # bundle
cd src-tauri && cargo test # parser / settings / language / release-allowlist unit tests
```

- Browser preview: `npm run dev`, then http://localhost:5173 — the footer notes it is mock data with no real switching.
- **Debugging the flag means going around `tauri dev`**: `npm run tauri dev -- -- -quick-switch` hands the argument to cargo, which fails on usage. Start vite first with `npm run dev`, then `cd src-tauri && cargo run -- -qs` — the debug build reads `devUrl`, so the frontend still hot-reloads. Note that a copy in `/Applications` competes for the same single-instance lock, so whichever of the two starts last quits itself.
- displayplacer is bundled in `src-tauri/binaries/` (universal binary + MIT license). At runtime `displayplacer_path()` prefers `Contents/Resources/binaries/` and falls back to PATH, so a development machine can also `brew install displayplacer` and use the system copy.
- **Do not run `cargo fmt`**: this repository is not rustfmt-clean, so one run reformats four or five unrelated files and mixes formatting noise into a feature commit. Hand-format new code to match its surroundings.
- Frontend code is **Options API** throughout — no `<script setup>`, no Composition API.

## Bundling and releasing

- `npm run tauri build` writes to `src-tauri/target/release/bundle/` (`.app` and `.dmg`). Binaries are universal (arm64 + x86_64) and ad-hoc signed, **not Apple-notarized**: the "Apple cannot verify" prompt on first launch is expected and takes one pass through System Settings → Privacy & Security → Open Anyway. If you do sign and distribute properly, `binaries/displayplacer` must be signed along with the app.
- **The version lives in three places**: `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` (plus the `display-switch` entry in `src-tauri/Cargo.lock`). Both the About row and the update check read `version` from `package.json`, so a release tag must be `v<version>` (for example `v1.0.0`) — otherwise the app compares itself against something it will never match.
- Create the release on both hosts: GitHub `WitsonX/DisplaySwitch` and Gitee `Witson/DisplaySwitch`. Gitee mirrors commits only — releases and attachments are not mirrored, so they have to be published there separately. Until at least one host has a release, "Check for updates" can only report "No releases yet".

## Known traps

- `CGDisplayScreenSize` returns the whole mirror group's size while mirroring; it cannot be used as a per-display size.
- Bilingual error strings are runtime templates, so `format!` inline interpolation does not work — use `tr_fill`.
- Tray click coordinates are physical pixels while window size is logical; one division by `scale_factor` is mandatory.
- In the settings view at 380×520 the content is 498px tall against 469px of viewport: even with an empty setup list you scroll a little to see the whole Quit button, and any saved setup overflows it anyway. That is inherent to a fixed window height; we did not compress approved spacing just to make the empty state fit.
- Mirrored displays show no star for favorites — their resolution follows the target, so starring something there could never take effect.
- **A second double-click drags the app back to the foreground**: Tauri only exposes `set_activation_policy` on `App`, so calling it in `setup` is already late — at check-in LaunchServices registers the process as a normal foreground app (`lsappinfo` shows `type="Foreground"`). Double-clicking again does not spawn a process; it delivers a reopen event, and AppKit restores the policy from that registration, so the Dock icon stays. Adding `LSUIElement` to `src-tauri/Info.plist` makes the registration itself an agent (`type="UIElement"`), which removes both the startup flash and the stuck icon. To inspect: read `activationPolicy` from `NSWorkspace.shared.runningApplications` (0=Regular / 1=Accessory).
- **The first click after the panel reappears can feel dead**: macOS by default spends the first click on an inactive window activating that window, and this panel hides on blur, so it often reappears unfocused. `acceptFirstMouse: true` in `tauri.conf.json` delivers that first click straight to the webview.
