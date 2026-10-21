# Documentation maintenance (fork)

The documentation site uses the Bun version declared in `package.json`.
From this directory:

```sh
bun install --frozen-lockfile
bun run test:image-security
bun run build
bun audit
```

The image check requires Sharp 0.35.5 or newer and tests PNG, WebP, AVIF and SVG
processing plus invalid-input rejection. The lockfile pins the complete resolved
dependency tree; updating only the direct package can leave vulnerable transitive
versions behind. Native application and database checks are separate; see
[the repository testing guide](../TESTING.md).

## Security status — 21 October 2026

The patched Sharp dependency and refreshed compatible lockfile resolve 25 of the
26 advisories reproduced by the prior lockfile's Bun audit, including
[GHSA-f88m-g3jw-g9cj](https://github.com/advisories/GHSA-f88m-g3jw-g9cj),
[GHSA-rgj7-g3m4-5g8c](https://github.com/advisories/GHSA-rgj7-g3m4-5g8c) and
[GHSA-wq5f-xc86-pv6w](https://github.com/advisories/GHSA-wq5f-xc86-pv6w).

One finding remains: `braces@3.0.3`,
[GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm),
via `starlight-llms-txt -> micromatch`. No patched braces release is published
at this check; the latest docs plugin retains that dependency. Do not suppress
this advisory or describe `bun audit` as clean. A successful build does not
certify that dependency or arbitrary untrusted glob patterns.

The workflow builds and checks this fork, but GitHub Pages deployment remains
restricted to the upstream repository. These changes do not create a new site,
publish upstream, replace installed Aven binaries or alter task data.
