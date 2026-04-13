# Research notes: golangci-lint, golangci-lint-action, and GitHub Actions

Sources: [golangci/golangci-lint-action](https://github.com/golangci/golangci-lint-action), [golangci/golangci-lint](https://github.com/golangci/golangci-lint), [GitHub workflow syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions).

---

## golangci-lint (`github.com/golangci/golangci-lint`)

- **What it is:** A fast Go linters runner that runs many linters in parallel, uses caching, supports YAML configuration, integrates with major IDEs, and bundles a large set of linters (100+).
- **Docs:** [https://golangci-lint.run](https://golangci-lint.run) (install, CI, configuration, migration guides).
- **Install:** Local and CI instructions are linked from the repo README ([local](https://golangci-lint.run/welcome/install/#local-installation), [CI](https://golangci-lint.run/welcome/install/#ci-installation)).

---

## golangci-lint-action (`github.com/golangci/golangci-lint-action`)

- **What it is:** The official GitHub Action from the golangci-lint authors. It runs `golangci-lint` in CI and surfaces issues (including [annotations](https://github.blog/2018-12-14-introducing-check-runs-and-annotations/) on the PR/commit UI when permissions allow).
- **Implementation:** JavaScript action (not Docker), so startup is fast; uses [@actions/cache](https://github.com/actions/toolkit/tree/HEAD/packages/cache) for `~/.cache/golangci-lint` and [@actions/tool-cache](https://github.com/actions/toolkit/tree/HEAD/packages/tool-cache) for the binary.
- **Recommendation:** Run lint in a **separate job** from `go test` etc., because jobs run in parallel.
- **Typical workflow shape:** `actions/checkout` → `actions/setup-go` (required from action **v4+**; `skip-go-installation` was removed) → `golangci/golangci-lint-action@v6` (or current major) with `with.version` pinning the linter (e.g. `v1.60`).
- **Notable inputs:** `version`, `install-mode` (`binary` default, `goinstall`, `none`), `args` (CLI passthrough), `working-directory` (monorepos), `only-new-issues` (needs GitHub API; default `github.token`, optional `pull-requests: read`), `skip-cache` / `skip-save-cache` / `cache-invalidation-interval`, `problem-matchers` (works with `colored-line-number` output).
- **Compatibility (from action README):** **v6.0.0+** removes the `annotations` option and the default output format `github-actions`. **v5+** removed `skip-pkg-cache` / `skip-build-cache` (Go cache handled by `setup-go`). **v4+** requires explicit `actions/setup-go` before the action.

---

## GitHub Actions workflow format (basics)

- **Location:** `.github/workflows/*.yml` or `*.yaml`.
- **Language:** YAML. Workflows define automated processes as one or more **jobs**.
- **Common top-level keys:**
  - `name` — shown under the repository Actions tab.
  - `run-name` — optional run title (can use expressions like `${{ github.actor }}`).
  - `on` — which [events](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows) trigger the workflow (`push`, `pull_request`, `schedule`, `workflow_dispatch`, `workflow_call`, etc.), with optional filters (`branches`, `paths`, `types`, …).
  - `permissions` — OAuth scopes for the `GITHUB_TOKEN` (e.g. `contents: read`, `checks: write`).
  - `env` — job- or workflow-level environment variables.
  - `jobs` — named jobs; each has `runs-on`, `needs`, `strategy`/`matrix`, `steps`, etc.
- **Jobs:** `runs-on` selects the runner (`ubuntu-latest`, `windows-latest`, …). Steps are sequential; each step can `uses: org/repo@ref` (action) or `run:` (shell command).
- **Actions:** `uses` references a repo tag/branch/SHA or a local path (`./.github/actions/...`). Inputs go under `with:`; secrets under `env:` or `secrets:`.
- **Expressions:** `${{ }}` for contexts (`github`, `env`, `secrets`, `matrix`, …).
- **Reusable workflows:** `on: workflow_call` with `inputs` / `secrets`; caller uses `jobs.<job_id>.uses: path-or-repo`.

---

## “GitHub Actions” output format vs annotations (golangci-lint + this action)

- The **`github-actions` output format** in golangci-lint was **deprecated as of v1.59 (2024-05-26)**. The official action **stopped supporting it in v6.0.0 (2024-05-07)** (see [release](https://github.com/golangci/golangci-lint-action/releases/tag/v6.0.0) and [discussion #5703](https://github.com/golangci/golangci-lint/discussions/5703)).
- **Current approach:** The action relies on golangci-lint’s **default text-style output** (documented as compatible with `--output.text.path=stdout` / the usual colored line-number style). [actions/setup-go](https://github.com/actions/setup-go) ships a **problem matcher** that recognizes this output; the action notes that `problem-matchers: true` forces embedded matchers and only applies with `colored-line-number` style.
- **Annotations in the Git UI:** To let the action write **checks** annotations, grant `checks: write` under `permissions` (in addition to `contents: read`). Annotations have GitHub platform limits (count, no Markdown in annotations, etc.—see the action README).

---

## Quick reference links

| Topic | URL |
| --- | --- |
| golangci-lint repo | https://github.com/golangci/golangci-lint |
| golangci-lint-action repo | https://github.com/golangci/golangci-lint-action |
| Workflow syntax (GitHub Docs) | https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions |
| Events that trigger workflows | https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows |
