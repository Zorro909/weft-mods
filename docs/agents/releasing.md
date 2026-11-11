# Releasing

`.github/workflows/release.yml` releases from every push to `main` through [weft-mod-action](https://github.com/Zorro909/weft-mod-action), whose README has the rules. A crate version is `X.Y.Z` for a full release or `X.Y.Z-rc.N` for a release candidate.

- A version without a release is released as it is when it reaches `main`.
- A mod that changed without a new version gets a candidate from CI, `X.Y.(Z+1)-rc.<run>.g<commit>`, set only in the build: nothing is committed.
- The workflows list the mod folders: a new mod goes into the `mods` of `release.yml`, `check.yml` and `index-pr.yml`. A shared library such as `weftkit/` stays out, and a change to it alone gives its users no candidate.
- `release.yml` keeps the releases section of the root `README.md`: leave what lies between its markers to it.

## The mod index

- The launcher lists the [Weft mod index](https://github.com/Weft-Rust/Weft-Mods-Index), which builds each mod itself from this repository at the commit proposed. It takes only full releases whose `Cargo.toml` holds that version, and shows the first sentence of `mod.<id>.description` in `en_us.lang` as the mod's summary.
- `index-pr.yml`, run by hand from the Actions tab, proposes the newest full releases, or the tags given, one pull request each. `release.yml` proposes each full release by itself once the repository variable `WEFT_INDEX_PR` is `true`. Both need the secret `WEFT_INDEX_TOKEN`.
- On a new Weft release the index builds each mod's newest version again by moving the one `weft_sdk` line of its `Cargo.toml` to the new tag. `ISSUE_LOG.md` says what that breaks for a mod with other Weft crates.

## Checks on a pull request

- The job names of `check.yml` are required checks in the ruleset of `main`: renaming or removing one blocks every pull request until the ruleset follows.
- `sonarqube.yml` sends clippy's findings and the test coverage to SonarQube Cloud (`sonar-project.properties`). Its gate wants 80% of new code covered: `testing.md`.
