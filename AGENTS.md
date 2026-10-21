# Aven fork maintenance

- Read this file and `README.md` before each task; consult `CONTRIBUTING.md` and the affected sections of `TESTING.md`.
- Verify source ownership, a clean Git baseline and the exact intended remote before editing or publishing. Preserve unrelated work.
- This fork is separate from upstream and from any installed application. A repository change does not authorize an upstream push, service restart, binary replacement or website deployment.
- Keep credentials, local configuration, task databases, generated output and dependency directories out of commits.
- For documentation dependency changes, preserve the Bun lockfile and use the Bun version declared in `docs/package.json`. Validate with `cd docs && bun install --frozen-lockfile && bun run build`; audit changed dependencies and check valid/invalid image inputs when changing image processing.
- Rust, database, sync and application changes must follow the relevant validation gates in `TESTING.md`; documentation-only checks do not establish native-runtime acceptance.
- Update affected maintenance documentation and release notes. Commit only scoped files and report exact checks, source references and unverified limits.
