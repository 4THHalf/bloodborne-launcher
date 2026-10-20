# New-version notifications

[← Documentation](README.md)

The launcher checks the stable releases of
[bloodborne-mac](https://github.com/bloodborne-native/bloodborne-mac/releases) against the version
built into the installed app (semantic version order; drafts and prereleases ignored; launcher
commits never notify). The app and launcher versions are kept equal for this.

- After profile setup it checks at startup and every six hours while visible.
- A newer release opens a changelog modal (the release's GitHub notes as Markdown: headings,
  lists, emphasis; no raw HTML, images or active links) once per release tag, and a banner with
  **Changelog** and dismiss. **View release** opens the release page; download and installation
  are manual, and nothing in the app or the saves is changed.
- **Advanced → Updates:** the installed version, the check status, **Check now**, and **Check for
  new versions automatically** (`check_updates`, on by default; manual checks always work).
- Offline, rate limits and bad responses show a retryable status and never block Play. Requests
  carry no account, token or player data; 10-second timeout, 512 KiB limit.

Rust fetches GitHub's latest-release endpoint (reqwest, rustls); TanStack Query shares the result
(six-hour freshness). Only validated stable tags on the app's release page are opened. Native
tests cover version order, build metadata, draft/prerelease exclusion, bad responses, missing
notes and link validation. Browser preview: `?update&lang=en` supplies a synthetic newer release.
