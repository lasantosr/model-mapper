# Contributing to model-mapper

Thank you for your interest in contributing to `model-mapper`! To maintain a high-quality codebase and automate our release pipeline, we follow a set of guidelines.

## 1. Conventional Commits & PR Titles

We use [Conventional Commits](https://www.conventionalcommits.org/) to organize our repository's change history and automate version bumps and changelog generation via `release-plz`.

**Note:** We use "Squash and Merge" for all Pull Requests. This means you do not need to format every single commit perfectly while working on your branch. **Only the PR title must follow the strict format.**

```text
type(scope): description
```

### Types

* `feat`: A new feature
* `fix`: A bug fix
* `docs`: Documentation changes
* `chore`: Maintenance, build tasks, dependencies, etc.
* `test`: Adding or correcting tests
* `refactor`: A code change that neither fixes a bug nor adds a feature
* `perf`: A code change that improves performance

### Scopes

The scope is optional but helps organize the changelog. If provided, use one of the following:

* `macro`: Changes to the procedural macro logic, AST parsing, and code generation (`model-mapper-macros`).
* `lib`: Changes to the public facade, trait definitions, and re-exports (`model-mapper`).
* `examples`: Improvements or additions to workspace examples.
* `deps`: Dependency updates.
* `ci`: CI/CD workflow and GitHub Actions updates.

### Breaking Changes

If a change introduces a breaking API change, you must append a `!` after the type/scope to alert the release tooling (e.g., `feat!:`, `fix(lib)!:`). You may also detail the breaking change in the PR description starting with `BREAKING CHANGE:`.

---

## 2. Local Development & Testing

Before opening a PR, ensure your code passes formatting, linting, and tests. We run these exact checks in CI.

**Format and Lint:**

```bash
cargo +nightly fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

**Run standard tests and examples:**

```bash
cargo test --all-targets --all-features
```
