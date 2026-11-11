# Releasing

How a mod's version becomes a release, and what CI checks on the way.

## Versions and releases

`.github/workflows/release.yml` publishes from every push to `main` with [weft-mod-action](https://github.com/Zorro909/weft-mod-action), whose README has the rules; the top of the workflow sums them up. Each release holds `<id>.zip`, the mod's libraries and `assets/`, for installing by hand. The launcher lists the [Weft mod index](https://github.com/Weft-Rust/Weft-Mods-Index) instead, which builds each mod itself from this repository at the commit proposed. A mod's crate version is `X.Y.Z` for a full release or `X.Y.Z-rc.N` for a release candidate.

- A version without a release is released as it is when it reaches `main`.
- A mod that changes without a new version gets a candidate from CI, `X.Y.(Z+1)-rc.<run>.g<commit>`, set only in the build: nothing is committed.
- The workflows list the mod folders: a new mod goes into the `mods` of `release.yml`, `check.yml` and `index-pr.yml`.
- A library the mods share, such as `weftkit/`, is no mod: it stays out of those lists, and a change to it alone gives its users no release candidate.

## The mod index

`.github/workflows/index-pr.yml`, run by hand from the Actions tab, proposes the newest full releases, or the tags given, to the mod index: one pull request each, at the release's commit, with the mod's `declare_mod!` deps. `release.yml` proposes each full release by itself once the repository variable `WEFT_INDEX_PR` is `true`. Both need the secret `WEFT_INDEX_TOKEN`. The index takes only full releases whose `Cargo.toml` holds that version, and shows the first sentence of `mod.<id>.description` from the mod's `en_us.lang` as its summary.

When a new Weft release comes out, the index builds each mod's newest version again against it by moving the one `weft_sdk` line in the mod's `Cargo.toml` to the new tag. `ISSUE_LOG.md` says what that breaks for a mod with other Weft crates, and how `tools/ci/check-weft.sh` keeps the mods here in step meanwhile.

## What CI checks

- `release.yml` keeps the releases section of the root `README.md` up to date: leave what lies between its markers to it.
- `check.yml` checks a pull request: rustfmt over every Rust file, every mod taking Weft at a release tag (`tools/ci/check-weft.sh`), `tools/check-mods.sh`, the tests of `tools/ci`, and a build of every mod on Windows, Linux and macOS with the action. Its job names are required checks in the ruleset of `main`: renaming or removing one blocks every pull request until the ruleset follows.
- `sonarqube.yml` runs clippy over every mod as `tools/check-mods.sh` does, measures the test coverage of the mods (`tools/ci/rust-coverage.sh`, with cargo-llvm-cov) and of `tools/ci` (`tools/ci/python-coverage.sh`), and sends both to SonarQube Cloud, set up in `sonar-project.properties`. Its quality gate wants 80% of new code covered; `testing.md` says how new code meets it.
