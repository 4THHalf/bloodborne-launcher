# New-version notifications

The development launcher checks stable GitHub releases of
[bloodborne-native/bloodborne-mac](https://github.com/bloodborne-native/bloodborne-mac/releases).
It compares the release tag with the version built into the installed app, using semantic
version precedence. Launcher repository commits do not trigger notifications. Drafts and
prereleases are excluded; build metadata does not make an otherwise equal version newer.

After profile setup, the launcher checks automatically on startup and every six hours while
visible. A newer release opens a changelog modal and shows a banner above the current tab. The modal
displays the release notes published on GitHub, with Markdown headings, lists and emphasis.
If no notes were provided, it says so. **View release** opens that release’s GitHub page in the
default browser. Read the notes, download the app and install it
manually; the launcher does not replace its own files or change saves.

Closing the modal leaves the banner available; its **Changelog** button reopens the notes.
The modal opens automatically once per new release tag during the window session. Dismiss on
the banner hides that release’s notification until the launcher window is reloaded. **Advanced →
Updates** shows the installed version and check status, and provides **Check now** and a release
link. Turn off **Check for new versions automatically** to stop automatic requests; manual
checks remain available. The preference is stored as `check_updates` in `launcher.json` and
requires no profile database migration. Existing installations default to enabled.

Offline connections, API limits and invalid responses show a retryable status in Advanced.
They do not block local setup, profiles or Play. Requests use no GitHub account or token and
send no save/profile data. They have a 10-second timeout and a 512 KiB response limit.

## Implementation and verification

Rust fetches the [GitHub latest-release endpoint](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)
with reqwest and rustls. TanStack Query shares the result between the banner and settings, with
a six-hour freshness period. Opening links uses Tauri's cross-platform opener plugin; only
validated stable version tags under the app repository's release page are accepted.

Five native tests cover numeric version ordering, equal build metadata, draft/prerelease
exclusion, failed/malformed responses, missing release notes and restricted release links. The normal `cargo test`
flow includes these tests. Browser preview `?update&lang=en` supplies a synthetic newer release
without network access or player data.

On 2026-10-13, all 36 native tests, Clippy with warnings denied, frontend type/lint/locale
checks and the production frontend build passed. An isolated headless browser check exercised
the newer banner, dismissal, installed version, release link, manual checks with automatic
checks disabled, offline status, retry, current version and no-release state. No player files
or screenshots were used. The live GitHub endpoint accepted the request headers and returned
`v0.5.1`. A packaged-app update-link check remains part of release acceptance.

The changelog addition passed all 37 native tests, Clippy, frontend checks and the production
build on 2026-10-13. An isolated browser check verified automatic opening, Markdown headings
and lists, the release button’s exact tag, dismissal/reopening, no reopening for the same tag,
a different release tag, and empty notes. Raw HTML, remote images and embedded links are not
rendered as active content; the modal’s release button remains the browser entry point.
