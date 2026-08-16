# Testing the in-app updater

Oden's in-app updater (`crates/oden-app/src/updater.rs`) talks to the GitHub
Releases API to check for and install new versions. To exercise that whole
flow ; check, download, "install", restart, and the post-update "what's
new" changelog popup ; without touching the real release channel or the
real running binary, use the fake update server at
`crates/oden-app/examples/fake_update_server.rs`.

It's a tiny local HTTP server that answers the same requests
`self_update` would send to `api.github.com`, but always reports one
canned fake release (`v99.99.99`) with a small placeholder asset instead
of a real Oden build.

## Quick start

In one terminal, start the fake server:

```sh
cargo run --example fake_update_server
```

It prints the port it's listening on (default `4067`, override with
`PORT=...`) and the exact command to run Oden against it. In a second
terminal:

```sh
ODEN_FAKE_UPDATE_SERVER=http://127.0.0.1:4067 cargo run -p oden
```

That's it ; Oden now treats `v99.99.99` as the latest available release
everywhere it would normally talk to GitHub.

## What's safe about this

The fake server makes the updater do a **real HTTP download** of a real
(tiny, placeholder) zip archive ; that part isn't mocked. What's
redirected is the *install* step: normally the updater replaces the
currently running executable in place. Whenever `ODEN_FAKE_UPDATE_SERVER`
is set, `install_update` instead points that replace step at a scratch
file under the OS temp dir (`oden-fake-update-install.bin`), never at the
real `oden`/`oden.exe`. You can click through "Install & Restart" in
Settings as many times as you like; nothing about your actual dev build
ever changes.

"Restart" still restarts the real, unmodified app (`cx.restart()`) ; it
just won't be running any different code, since nothing was actually
replaced.

## What you can test with it

- **Settings → Updates → Check for Updates** ; reports `v99.99.99`
  available, with a "View Release" / "Install & Restart" action depending
  on the detected update channel.
- **Install & Restart** ; runs the real download-and-replace flow against
  the scratch file described above, then shows "Updated to v99.99.99 ;
  restart to apply" with a working restart button. The button shows a real
  progress bar driven by actual bytes received via `Content-Length` ; not
  a simulation ; `install_update`  reads the download in 64KB chunks and 
  reports progress after every one.
  The fake asset is padded to `FAKE_ASSET_SIZE` (4MB) and the download
  endpoint paces delivery to take about `THROTTLE_TOTAL` (6s) in total,
  scaled by bytes actually sent per `read()` call rather than a fixed
  per-call delay ; real downloads over localhost would otherwise arrive in
  a handful of calls, too fast to watch the bar move, and a fixed-delay
  scheme breaks badly if the number of `read()` calls doesn't match what it
  assumed. Bump `THROTTLE_TOTAL`
  if you want it longer, or shrink it for a fast loop while iterating on
  something else ; the pacing scales correctly either way.
- **The "what's new" greeting popup** ; normally shown once, the first
  time Oden runs after landing on a new version, with changelog text
  fetched from that version's GitHub Release page. With
  `ODEN_FAKE_UPDATE_SERVER` set, this fires **on every startup**,
  regardless of the last-seen-version bookkeeping in Settings, using the
  fake server's canned changelog. The changelog body is rendered through
  Oden's own markdown renderer (`comrak-gpui`), so this also doubles as a
  quick way to sanity-check that renderer against new markdown.

  Since the startup popup only ever fires once, right when the window
  opens, it's easy to miss while testing. There's also a **"Preview
  'What's New'" button** in Settings → Updates that triggers the exact
  same dialog on demand, any time ; use that instead of restarting the app
  if you just want to look at it again. It works with or without
  `ODEN_FAKE_UPDATE_SERVER` set (in real mode it shows the *currently
  running* version's own release notes).
- **Update channel detection** ; the fake server doesn't change which
  channel Oden thinks it's running on (`UpdateChannel::detect`, based on
  `FLATPAK_ID` / `SNAP` / `APPIMAGE` env vars). To test the "packaged, no
  self-update" path, set the relevant env var alongside
  `ODEN_FAKE_UPDATE_SERVER`, e.g.:

  ```sh
  ODEN_FAKE_UPDATE_SERVER=http://127.0.0.1:4067 SNAP=1 cargo run -p oden
  ```

  You should see a "View Release" link instead of an install button, since
  packaged channels never self-update.

## Endpoints it implements

Just enough of the GitHub Releases API surface for `self_update` to work
against it:

| Route | Mirrors |
|---|---|
| `GET /repos/:owner/:repo/releases/latest` | Used by the "check for updates" and "what's new" (fake mode) flows |
| `GET /repos/:owner/:repo/releases` | Used by the "install" flow |
| `GET /download/<asset-name>` | Serves the placeholder zip asset |

The reported asset name is styled after the real release naming scheme for
your platform (e.g. `oden-linux-x64_99.99.99.zip`), but since this server
always reports exactly one asset, the updater takes it unconditionally
instead of running the prefix/extension matching it uses against a real
release's several platform assets - see `self_update_asset_hint()` in
`src/updater.rs`.

## Going back to real GitHub

Just unset `ODEN_FAKE_UPDATE_SERVER` (or don't set it) and run Oden
normally. Nothing about the fake-server code path is compiled into
release builds' behavior by default ; it's purely an env var check in
`updater.rs`, off unless you opt in.
