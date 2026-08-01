---
name: reference_android_manifest_needs_internet
description: An app that fetches over the web needs android.permission.INTERNET in its manifest.yaml, and template/manifest.yaml does not carry it
type: reference
---

An android app that reads anything over the web needs `uses_permission` with
`android.permission.INTERNET` in its `manifest.yaml`. Without it every `ehttp`
fetch fails and reports nothing: the app draws, and the buttons that fetch do
nothing at all.

`template/manifest.yaml` does not carry it, because the template fetches
nothing. So a new app copied from the template that fetches will repeat this.
The apps that fetch are `apps/viewer` and the ten in `apps/rendering/` whose
`systems/model.rs` reads a Khronos sample model.

```yaml
    uses_permission:
      - name: android.permission.INTERNET
```

Confirm it landed with `adb shell dumpsys package com.nightshade.<name>` and
read the requested permissions.

Related: [[reference_viewer_layout_rebuilds_on_width]]
