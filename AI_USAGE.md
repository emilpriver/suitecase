# AI-assisted development

Using a large language model (LLM) or similar assistant to draft patches, explore refactors, or write tests for this repository is **fine**. The maintainer and reviewers still need a **human** who stands behind the work: **assistants are tools, not authors.**

The expectations below are our project rules, written in the same spirit as the Linux kernel’s [`Documentation/process/coding-assistants.rst`](https://raw.githubusercontent.com/torvalds/linux/refs/heads/master/Documentation/process/coding-assistants.rst), adapted here for this **MIT**-licensed Rust crate and a typical GitHub PR workflow.

## Expectations

**Follow the normal process.** AI-assisted changes should go through the same path as any other contribution: project conventions, code style, and how pull requests are described and reviewed. Use the README, existing code, and any contributor docs as the source of truth.

**Understand the change and be able to justify it.** You should grasp every meaningful part of what you submit—behavior, tradeoffs, and how it fits the design. Reviewers may ask *why* something was done; answers like “the model suggested it” are not enough. Tie decisions to project goals: correctness, API stability, performance, clarity, or maintenance cost. If you cannot explain it in review, read and simplify before opening a PR.

**You certify the contribution.** Only a **human** can certify the [Developer Certificate of Origin](https://developercertificate.org/) (DCO) where your workflow uses `Signed-off-by` or equivalent. **AI agents must not add `Signed-off-by` lines.** You are responsible for reviewing all AI-generated code, ensuring it meets licensing and project rules, and taking **full responsibility** for the submission. Do not let tooling imply that an automated system signed off on behalf of a person.

**Avoid unnecessary complexity.** LLMs often add more abstraction, indirection, or generality than the task needs; that makes changes easier to **question, rewrite, or reject** in review. Prefer the smallest change that solves the problem and matches surrounding style.

**Verify locally.** Run tests, `clippy`, and formatting as appropriate. Generated code can compile and still be wrong or brittle.

**Licensing.** Contributions must comply with this repository’s license (see [`LICENSE`](LICENSE)). Do not submit code you are not allowed to distribute under those terms. When in doubt, ask before opening a PR.

## Optional attribution (`Assisted-by`)

You may document how a change was produced with an **`Assisted-by`** line in a commit message or PR description:

```text
Assisted-by: AGENT_NAME:MODEL_VERSION [TOOL1] [TOOL2]
```

* `AGENT_NAME` — assistant or product name.  
* `MODEL_VERSION` — specific model or version.  
* `[TOOL1] [TOOL2]` — optional **specialized** tools (e.g. linters, codemods) relevant to the change.

Do **not** list ordinary tools such as git, `rustc`, `cargo`, or editors.

Example:

```text
Assisted-by: ExampleAssistant:model-2025-01 clippy rustfmt
```

Attribution is optional unless maintainers ask for it on a given change.
