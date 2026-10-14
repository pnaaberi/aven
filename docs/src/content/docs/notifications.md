---
title: Notifications
description: Enable desktop notifications and allow them during Focus.
---

Aven sends a desktop notification when background AI task creation succeeds after you close the TUI.

## macOS

On macOS, Homebrew and the install script include an `Aven.app` bundle so notifications appear as **Aven**, with its own icon and notification settings.

<div class="notification-example-light">
  <img src="/macos-notification-light.webp" alt="Illustration of an Aven notification confirming the task Review the homepage copy was added" width="440" height="169" />
</div>
<div class="notification-example-dark">
  <img src="/macos-notification-dark.webp" alt="Illustration of an Aven notification confirming the task Review the homepage copy was added" width="440" height="169" />
</div>

Choose **Allow** when macOS first asks. To change this later, open **System Settings → Notifications → Aven**.

### Allow notifications during Focus

Open **System Settings → Focus**, select your Focus, and add **Aven** to its allowed apps. Repeat for each Focus where you want notifications.

### Notifications appear as Terminal

Nix and bare-binary installs send notifications as **Terminal**. Use Terminal’s notification and Focus settings instead.
