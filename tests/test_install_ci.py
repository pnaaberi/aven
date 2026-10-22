"""Exercise CI-install trust without GitHub, artifacts or a native installation."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[1]
SHA = "a" * 40
OTHER_SHA = "b" * 40
WORKFLOW = (ROOT / ".github/workflows/install.yml").read_text()
VALIDATION = WORKFLOW.split("      - name: Validate commit\n", 1)[1].split(
    "      - uses:", 1
)[0]
GUARD = textwrap.dedent(VALIDATION.split("        run: |\n", 1)[1])

# Every external authority is a local stub. Stop at watch, before any artifact
# download/extraction or native replacement, even for valid dispatch fixtures.
STUB = r'''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import sys

name = Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ["FIXTURE_LOG"], "a") as output:
    output.write(json.dumps({"tool": name, "args": args}) + "\n")
if name == "uname":
    print("Darwin" if args == ["-s"] else "arm64")
elif name == "git":
    if args[:1] == ["-C"]:
        args = args[2:]
    if args == ["rev-parse", "--show-toplevel"]:
        print(os.environ["FIXTURE_REPO"])
    elif args == ["status", "--porcelain"]:
        print(" M fixture" if os.environ.get("FIXTURE_DIRTY") else "")
    elif args == ["symbolic-ref", "--quiet", "--short", "HEAD"]:
        if os.environ.get("FIXTURE_DETACHED"):
            sys.exit(1)
        print(os.environ["FIXTURE_BRANCH"])
    elif args == ["rev-parse", "HEAD"]:
        print(os.environ["FIXTURE_SHA"])
    elif args[:1] != ["push"]:
        sys.exit(91)
elif name == "gh":
    if args[:2] == ["repo", "view"]:
        print("example/repo")
    elif args[:2] == ["workflow", "run"]:
        if os.environ.get("FIXTURE_DISPATCH_FAIL"):
            sys.exit(19)
        print("https://github.com/example/repo/actions/runs/123")
    elif args[:2] == ["run", "watch"]:
        sys.exit(23)
    else:
        sys.exit(92)
else:
    sys.exit(93)
'''


class WorkflowTrust(unittest.TestCase):
    def guard(self, requested=SHA, dispatched=SHA, ref_type="branch"):
        return subprocess.run(
            ["bash", "-euo", "pipefail", "-c", GUARD],
            env={"PATH": os.defpath, "COMMIT_SHA": requested,
                 "DISPATCH_SHA": dispatched, "REF_TYPE": ref_type},
            capture_output=True, text=True, timeout=5,
        )

    def test_matching_branch_revision_is_allowed(self):
        self.assertEqual(self.guard().returncode, 0)

    def test_valid_but_different_revision_and_moved_branch_fail_closed(self):
        self.assertNotEqual(self.guard(dispatched=OTHER_SHA).returncode, 0)

    def test_tag_context_is_rejected_even_with_matching_revision(self):
        self.assertNotEqual(self.guard(ref_type="tag").returncode, 0)

    def test_malformed_values_are_data_not_shell(self):
        for value in ("", "a" * 39, "A" * 40, "a" * 41, "$(exit 0)"):
            with self.subTest(value=value):
                self.assertNotEqual(self.guard(requested=value).returncode, 0)

    def test_validation_precedes_privileged_work_and_checkout_is_context_bound(self):
        self.assertIn("DISPATCH_SHA: ${{ github.sha }}", VALIDATION)
        self.assertIn("REF_TYPE: ${{ github.ref_type }}", VALIDATION)
        self.assertNotIn("${{", GUARD)
        checkout = WORKFLOW.split("      - uses: actions/checkout@", 1)[1].split(
            "      - uses:", 1
        )[0]
        self.assertIn("ref: ${{ github.sha }}", checkout)
        self.assertNotIn("inputs.commit_sha", checkout)
        self.assertLess(WORKFLOW.index("- name: Validate commit"),
                        WORKFLOW.index("- uses: actions/checkout@"))
        self.assertIn("save-if: ${{ github.ref == 'refs/heads/main' }}", WORKFLOW)


class InstallerDispatch(unittest.TestCase):
    def run_installer(self, branch="feature/ref-trust", **flags):
        with tempfile.TemporaryDirectory(prefix="aven-install-ci-fixture-") as tmp:
            directory = Path(tmp)
            binaries = directory / "bin"
            binaries.mkdir()
            repository = directory / "repo"
            repository.mkdir()
            home = directory / "home"
            home.mkdir()
            for tool in ("git", "gh", "uname"):
                path = binaries / tool
                path.write_text(STUB)
                path.chmod(0o755)
            log = directory / "calls.jsonl"
            env = {"PATH": str(binaries) + os.pathsep + os.defpath,
                   "HOME": str(home), "CARGO_HOME": str(directory / "cargo"),
                   "CARGO_INSTALL_ROOT": str(directory / "install"),
                   "FIXTURE_LOG": str(log), "FIXTURE_REPO": str(repository),
                   "FIXTURE_BRANCH": branch, "FIXTURE_SHA": SHA, **flags}
            result = subprocess.run(
                ["bash", str(ROOT / "scripts/install-ci")], cwd=repository,
                env=env, capture_output=True, text=True, timeout=5,
            )
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            self.assertFalse((directory / "install/bin/aven").exists())
            self.assertFalse((directory / "cargo/bin/aven").exists())
            self.assertFalse(any(call["tool"] == "gh" and call["args"][:2]
                                 == ["run", "download"] for call in calls))
            return result, calls

    def test_pushed_branch_is_quoted_and_never_falls_back_to_main(self):
        for branch in ("main", "feature/ref-trust", "topic/a'b"):
            with self.subTest(branch=branch):
                result, calls = self.run_installer(branch)
                self.assertEqual(result.returncode, 23)  # simulated watch failure
                dispatches = [call["args"] for call in calls if call["tool"] == "gh"
                              and call["args"][:2] == ["workflow", "run"]]
                self.assertEqual(len(dispatches), 1)
                args = dispatches[0]
                self.assertEqual(args[args.index("--ref") + 1], branch)
                self.assertIn("commit_sha=" + SHA, args)
                pushes = [call["args"] for call in calls if call["tool"] == "git"
                          and "push" in call["args"]]
                self.assertEqual(pushes[0][-1], "HEAD:refs/heads/" + branch)

    def test_dispatch_failure_does_not_watch_download_or_install(self):
        result, calls = self.run_installer(FIXTURE_DISPATCH_FAIL="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(call["tool"] == "gh" and call["args"][:2]
                             == ["run", "watch"] for call in calls))

    def test_dirty_or_detached_source_does_not_dispatch(self):
        for flag in ("FIXTURE_DIRTY", "FIXTURE_DETACHED"):
            with self.subTest(flag=flag):
                result, calls = self.run_installer(**{flag: "1"})
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any(call["tool"] == "gh" and call["args"][:2]
                                     == ["workflow", "run"] for call in calls))


class SourceBuildContracts(unittest.TestCase):
    def test_arm_musl_c_atomics_do_not_require_outlined_helpers(self):
        dockerfile = (ROOT / "Dockerfile").read_text()
        self.assertIn(
            "ENV CFLAGS_aarch64_unknown_linux_musl=-mno-outline-atomics", dockerfile
        )

    def test_nix_release_reference_is_explicitly_upstream(self):
        workflow = (ROOT / ".github/workflows/nix.yml").read_text()
        self.assertIn("gh release view --repo raine/aven", workflow)
        self.assertIn('gh release download "$version" --repo raine/aven', workflow)

    def test_full_builds_cover_pr_main_and_preserve_explicit_dispatch(self):
        for name in ("ci.yml", "container.yml"):
            workflow = (ROOT / ".github/workflows" / name).read_text()
            self.assertIn("  push:\n    branches: [main]\n", workflow)
            self.assertIn("  pull_request:", workflow)
            self.assertIn("  workflow_dispatch:", workflow)


if __name__ == "__main__":
    unittest.main()
