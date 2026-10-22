# Documentation maintenance (fork)

The documentation site uses the Bun version declared in `package.json`.
From this directory:

```sh
bun install --frozen-lockfile
bun run test:image-security
bun run test:matcher-security
bun run build
bun audit
```

The image check requires Sharp 0.35.5 or newer and tests PNG, WebP, AVIF and SVG
processing plus invalid-input rejection. The lockfile pins the complete resolved
dependency tree; updating only the direct package can leave vulnerable transitive
versions behind. Native application and database checks are separate; see
[the repository testing guide](../TESTING.md).

## Security status — 22 October 2026

The patched Sharp dependency and compatible lockfile refresh, followed by removal
of the vulnerable unused brace-expansion dependency, resolve all 26 advisories
reproduced by the prior lockfile's Bun audit, including
[GHSA-f88m-g3jw-g9cj](https://github.com/advisories/GHSA-f88m-g3jw-g9cj),
[GHSA-rgj7-g3m4-5g8c](https://github.com/advisories/GHSA-rgj7-g3m4-5g8c) and
[GHSA-wq5f-xc86-pv6w](https://github.com/advisories/GHSA-wq5f-xc86-pv6w).

The docs plugin uses only `micromatch.isMatch`, which delegates to Picomatch.
A pinned, explicit `micromatch -> npm:picomatch@4.0.4` override supplies that same
limited API without pulling in unpatched `braces` (GHSA-vfj7-8cjw-p6xm). This is
not a drop-in replacement for the rest of Micromatch's API: the regression checks
the plugin's exact API usage, representative glob behavior and over-length input
rejection. Re-review the override if a future consumer needs other methods.
This adoption preserves the pre-override llms.txt/full/small bytes. No advisory is
ignored or suppressed; the hosted workflow runs the matcher test and `bun audit`.

The workflow builds and checks this fork, but GitHub Pages deployment remains
restricted to the upstream repository. These changes do not create a new site,
publish upstream, replace installed Aven binaries or alter task data.
