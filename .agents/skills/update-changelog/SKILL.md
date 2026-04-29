---
name: update-changelog
description: "Read this skill before updating changelogs"
---

# Changelog Rules

## Location

`CHANGELOG.md` in the project root.

## Format

Use these sections under `## [Unreleased]`:

### Breaking Changes - API changes requiring migration
### Added - New features
### Changed - Changes to existing functionality
### Fixed - Bug fixes
### Removed - Removed features

## Rules

1. Before adding entries, read the full `[Unreleased]` section to see which subsections already exist
2. New entries ALWAYS go under `## [Unreleased]` section
3. Append to existing subsections (e.g., `### Fixed`), do not create duplicates
4. NEVER modify already-released version sections (e.g., `## 0.0.7`)
5. Each version section is immutable once released

## Attribution

### Internal changes (from issues)

```markdown
* Fixed foo bar ([#123](https://github.com/<owner>/suitecase/issues/123))
```

### External contributions

```markdown
* Added feature X ([#456](https://github.com/<owner>/suitecase/pull/456) by [@username](https://github.com/username))
```

## Releasing

### Version semantics

- **patch**: Bug fixes and new features
- **minor**: API breaking changes
- No major releases

### Steps

1. **Update CHANGELOG**: Ensure all changes since last release are documented in the `[Unreleased]` section of `CHANGELOG.md`

2. **Run release script** (if available):
   ```bash
   cargo release patch    # Fixes and additions
   cargo release minor    # API breaking changes
   ```

3. The release process handles: version bump in `Cargo.toml`, CHANGELOG finalization, commit, git tag, and publish to crates.io.

## Git Rules

### Committing

- ONLY commit files YOU changed in THIS session
- ALWAYS include `fixes #<number>` or `closes #<number>` in the commit message when there is a related issue or PR
- NEVER use `git add -A` or `git add .`
- ALWAYS use `git add <specific-file-paths>` listing only files you modified
- Before committing, run `git status` and verify you are only staging YOUR files

### Forbidden Git Operations

These commands can cause problems:

- `git reset --hard` - destroys uncommitted changes
- `git checkout .` - destroys uncommitted changes
- `git clean -fd` - deletes untracked files
- `git stash` - stashes ALL changes
- `git add -A` / `git add .` - stages unintended changes
- `git commit --no-verify` - bypasses required checks

### Safe Workflow

```bash
# 1. Check status first
git status

# 2. Add ONLY your specific files
git add CHANGELOG.md
git add src/suite/mod.rs

# 3. Commit
git commit -m "fix: description"

# 4. Push
git push
```
