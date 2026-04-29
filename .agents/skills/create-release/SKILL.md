# Skill: create-release

Create a draft GitHub release from the `[Unreleased]` section of `CHANGELOG.md`, matching the pi-mono release format.

## Workflow

### 1. Read the changelog

Read `CHANGELOG.md` and extract the `[Unreleased]` section. If no `[Unreleased]` section exists, stop and tell the user to add one first.

### 2. Determine version

Ask the user what version to release (patch, minor, or specific version). 

- **patch**: Bug fixes and new features
- **minor**: API breaking changes

Bump the version in `Cargo.toml` accordingly. The release tag will be `v{version}`.

### 3. Format release notes

Parse the `[Unreleased]` subsections and format them into release notes:

```markdown
### New Features

* Description of feature ([#123](https://github.com/{owner}/{repo}/pull/123) by [@username](https://github.com/username))

### Added

* Description ([#456](https://github.com/{owner}/{repo}/pull/456))

### Changed

* Description ([#789](https://github.com/{owner}/{repo}/issues/789))

### Fixed

* Description ([#101](https://github.com/{owner}/{repo}/pull/101) by [@username](https://github.com/username))
```

#### Section mapping

| Changelog section | Release section |
|-------------------|-----------------|
| `### Breaking Changes` | `### Breaking Changes` |
| `### Added` | `### Added` |
| `### Changed` | `### Changed` |
| `### Fixed` | `### Fixed` |
| `### Removed` | `### Removed` |

#### Attribution rules

- **Internal changes** (from issues): `description ([#123](https://github.com/{owner}/{repo}/issues/123))`
- **External contributions** (from PRs with author): `description ([#456](https://github.com/{owner}/{repo}/pull/456) by [@username](https://github.com/username))`
- If no issue/PR number is found in the entry, use the description only

#### Entry formatting

Each entry is a bullet point (`* `) with the description followed by attribution in parentheses.

### 4. Get repository info

Run `gh repo view --json nameWithOwner,owner` to get the repository owner and name for URL construction.

### 5. Update CHANGELOG

Replace `## [Unreleased]` with `## {version} — {date}` where date is today in `YYYY-MM-DD` format.

### 6. Verify build

Run `cargo check` to ensure the project builds cleanly. If it fails, stop and tell the user.

### 7. Commit changes

Run:

```bash
git add CHANGELOG.md Cargo.toml
git commit -m "chore: release v{version}"
```

### 8. Create draft release

Run:

```bash
gh release create v{version} --draft --title "v{version}" --notes "{formatted_notes}"
```

### 9. Confirm

Output the draft release URL: `https://github.com/{owner}/{repo}/releases/edit/v{version}`

## Rules

1. **NEVER** publish a release directly — always create as draft first
2. **NEVER** modify already-released version sections in CHANGELOG.md
3. **ALWAYS** read the full `[Unreleased]` section before formatting
4. **ALWAYS** bump version in `Cargo.toml` before creating the release
5. **ALWAYS** run `cargo check` and verify it passes before committing
6. If the `[Unreleased]` section is empty, stop and tell the user
7. Skip empty subsections in the release notes (don't include sections with no entries)
8. Commit both `CHANGELOG.md` and `Cargo.toml` with message `chore: release v{version}`

## Git Rules

### Committing

- ONLY commit files YOU changed in THIS session
- ALWAYS include `fixes #<number>` or `closes #<number>` in the commit message when there is a related issue or PR
- NEVER use `git add -A` or `git add .`
- ALWAYS use `git add <specific-file-paths>` listing only files you modified

### Forbidden Git Operations

- `git reset --hard`
- `git checkout .`
- `git clean -fd`
- `git stash`
- `git add -A` / `git add .`
- `git commit --no-verify`

## Example

Given this `[Unreleased]` section:

```markdown
## [Unreleased]

### Added

* New mock expectations API ([#11](https://github.com/emilpriver/suitecase/pull/11) by [@contributor](https://github.com/contributor))

### Fixed

* Fixed test sync issue ([#14](https://github.com/emilpriver/suitecase/issues/14))
```

The release notes become:

```markdown
### Added

* New mock expectations API ([#11](https://github.com/emilpriver/suitecase/pull/11) by [@contributor](https://github.com/contributor))

### Fixed

* Fixed test sync issue ([#14](https://github.com/emilpriver/suitecase/issues/14))
```
