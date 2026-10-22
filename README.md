<h1>
  <a href="https://aventasks.dev#gh-light-mode-only"><img src="docs/src/assets/aven-logo-grape-violet-light.svg" width="48" height="48" align="absmiddle" alt="Aven logo"></a>
  <a href="https://aventasks.dev#gh-dark-mode-only"><img src="docs/src/assets/aven-logo-grape-violet.svg" width="48" height="48" align="absmiddle" alt="Aven logo"></a>
  aven
</h1>

`aven` is a local-first task manager for power users and agents. It gives you one overview across
projects, task capture from wherever work appears, first-class agent workflows, workspace isolation,
and a polished terminal UI.

It is currently under active development, but already works really well and I use it as my daily
driver.

If you've tried aven, feedback is welcome! Please
[open an issue](https://github.com/raine/aven/issues) to share it.

If you find aven useful, consider sharing it with others who might benefit from it.

Docs: <https://aventasks.dev/getting-started/>

<p>
  <a href="meta/tui.webp"><img src="meta/tui.webp" width="74%" alt="aven TUI showing the queue view across workspace projects"></a>
  <a href="meta/ios-light.webp#gh-light-mode-only"><img src="meta/ios-light.webp" width="24%" alt="Aven iOS queue in light mode with the TUI demo tasks, priorities, and project labels in an iPhone frame"></a>
  <a href="meta/ios-dark.webp#gh-dark-mode-only"><img src="meta/ios-dark.webp" width="24%" alt="Aven iOS queue in dark mode with the TUI demo tasks, priorities, and project labels in an iPhone frame"></a>
</p>

*Your tasks, away from the terminal. Aven for iOS and Android is coming soon.*

## Why aven?

The CLI is agent-first, while the [power-user TUI](https://aventasks.dev/tui/) gives humans a
keyboard-driven path to every action.

The [queue view](https://aventasks.dev/concepts/#queue) brings together tasks from every project
and shows what needs action, what is blocked, and what to focus on next.

Aven keeps tasks in a local SQLite database instead of tracked files inside each project repo. You
and your agents can capture and update tasks offline, independent of git state, branches, worktrees,
or checkouts. If you need the same tasks on more than one device, you can
[sync them through a server you control](https://aventasks.dev/sync/setup/). Sync is end-to-end
encrypted, so the server cannot read your tasks. No server to run?
[Aven Cloud](https://cloud.aventasks.dev/) is in free preview.

Repositories map to projects by default. Aven creates a project when you add its first task, and
gives each task a short Jira/Linear-style ID such as `APP-7KQ9`. Aven can generate them without a
server connection, and the project prefix shows where each task belongs.

[Workspaces](https://aventasks.dev/organize-tasks/#separate-work-with-workspaces) keep personal and
work tasks in separate views while sharing one database. You can map a directory such as `~/work` to
a workspace. When you run aven from that directory, it selects the workspace automatically.

Markdown descriptions, append-style notes, and
[image attachments](https://aventasks.dev/tui/#image-attachments) keep task context available to
you and your agents. Copy a complete task report as Markdown or
[publish it as an unlisted GitHub gist](https://aventasks.dev/tui/#publish-a-task-report-as-a-github-gist)
when you want to share that context. You can capture work through natural-language input, a tmux
popup, or agent-friendly commands, then schedule it to appear when it needs your attention.

Inspired by Taskwarrior. See [aven and Taskwarrior](https://aventasks.dev/taskwarrior/).

## Some kind words from early users

- *"This is by far the best TUI application I've come across in months."* -
  [Edke on GitHub](https://github.com/raine/aven/issues/5#issuecomment-5048980472)
- *"I am really impressed by the functionality at this early stage. Became a
  default tool for me already and serves also as shared memory and persistent
  learnings for my agents."* -
  [André Schemaitat on LinkedIn](https://www.linkedin.com/posts/schemaitat_tool-recommendation-for-local-task-management-share-7486060060650016768-5WRG/)
- *"I'm having a blast using the TUI, UX is perfect."* -
  [xe6 on GitHub](https://github.com/raine/aven/pull/19#issuecomment-5455250814)
- *"It's a genuinely beautiful TUI and a really promising task manager."* -
  [sudoremo on GitHub](https://github.com/raine/aven/issues/3)
- *"I use it daily and it has been great."* -
  [seungjuchoi on GitHub](https://github.com/raine/aven/pull/11)

## Quick start

Install with the release script:

```sh
curl -fsSL https://raw.githubusercontent.com/raine/aven/main/scripts/install | bash
```

Or install with Homebrew:

```sh
brew install raine/aven/aven
```

Open Aven:

```sh
aven
```

Or try the demo:

```sh
aven demo
```

Every demo run starts from the same sample data, and changes are discarded on exit.

See [Getting started](https://aventasks.dev/getting-started/) for first-run usage.

## Documentation

- [What is aven?](https://aventasks.dev/)
- [Getting started](https://aventasks.dev/getting-started/)
- [Concepts](https://aventasks.dev/concepts/)
- [Using the TUI](https://aventasks.dev/tui/)
- [Organizing tasks](https://aventasks.dev/organize-tasks/)
- [Scheduling tasks](https://aventasks.dev/schedule-tasks/)
- [Recurring tasks](https://aventasks.dev/recurring-tasks/)
- [Work with agents](https://aventasks.dev/agents/)
- [Sync across devices](https://aventasks.dev/sync/setup/)
- [Back up and restore](https://aventasks.dev/backups/)
- [Command reference](https://aventasks.dev/command-reference/)
- [Configuration](https://aventasks.dev/configuration/)

For this fork's documentation dependency checks and known audit limits, see
[documentation maintenance](docs/README.md).


### Fork CI installer trust

`scripts/install-ci` dispatches the same attached branch it pushes. The build
accepts branch refs only and requires the requested SHA to equal GitHub's dispatch
revision before checkout, toolchain setup, cache access or build. Checkout and
artifact identity use that immutable context SHA; automatic cache saves are
main-only. Branches may still read default caches under GitHub's normal policy.
Moved refs, mismatches and tags fail closed; there is no fallback to main.
A feature branch must contain the matching workflow. Arbitrary historical-SHA
builds now require a matching authorized branch. Artifact SHA/checksum checks
remain in place. This fork change does not update upstream or installed Aven.

Run `python3 tests/test_install_ci.py` for hermetic guard/dispatch regressions;
these use stub authorities and never perform a native installation.


### Fork native dependency maintenance

The lockfile keeps TLS/QUIC/certificate, event-listener and LRU dependencies on
patched compatible releases. Non-macOS notifications use notify-rust's supported
vendored D-Bus backend instead of the unmaintained standalone zvariant stack;
the Notification API and existing Windows/macOS notification stacks stay unchanged. Building the
vendored library requires a C compiler; runtime delivery still uses the user's
existing session D-Bus. Keep upstream library notices for binary redistribution.
`cargo audit --deny warnings` and the existing deny/license gate must remain clear. The isolated
missing-bus regression proves error handling, not on-screen notification display.
This is fork source maintenance, not installed-binary or upstream adoption.


Static ARM musl builds use target-specific inline C atomics for the vendored
D-Bus library, avoiding unavailable GCC outline helpers without changing other
target flags. Full/container checks run on PRs, main and explicit dispatch rather
than twice for each feature push. Nix release-flake validation explicitly reads
upstream `raine/aven` release assets; a fork needs no invented local release.
