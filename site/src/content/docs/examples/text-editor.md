---
title: "text-editor"
description: Text editor demonstrating file dialogs, clipboard, and context menus
slug: examples/text-editor
---

A simple text editor demonstrating file operations and UI integration.

## Overview

This example shows:
- File open/save dialogs
- Full file system access patterns
- Clipboard read/write
- Context menus
- Window title updates

## Features

- Open and save text files
- Standard edit operations (cut, copy, paste)
- File change detection (unsaved indicator)
- Right-click context menu

## Running

```bash
forge dev examples/text-editor
```

## Capabilities

```toml
[capabilities.fs]
read = ["~/*", "./*"]   # Home directory and current directory
write = ["~/*", "./*"]

[capabilities.sys]
clipboard = true
notifications = true

[capabilities.channels]
allowed = ["*"]
```

## Key Patterns

### File Dialogs

```typescript
import { showOpenDialog, showSaveDialog } from "runtime:window";
import { readTextFile, writeTextFile } from "runtime:fs";

// Open file
const result = await showOpenDialog({
  title: "Open File",
  filters: [
    { name: "Text Files", extensions: ["txt", "md"] },
    { name: "All Files", extensions: ["*"] }
  ]
});

if (result.filePaths.length > 0) {
  const content = await readTextFile(result.filePaths[0]);
  // Update editor...
}

// Save file
const saveResult = await showSaveDialog({
  title: "Save File",
  defaultPath: currentFile || "untitled.txt"
});

if (saveResult.filePath) {
  await writeTextFile(saveResult.filePath, editorContent);
}
```

### Window Title

```typescript
import { setWindowTitle } from "runtime:window";

// Show filename and unsaved indicator
const title = `Forge Editor - ${filename}${hasChanges ? " *" : ""}`;
await setWindowTitle(windowId, title);
```

### Clipboard

```typescript
import { readClipboard, writeClipboard } from "runtime:sys";

// Cut
const selected = getSelection();
await writeClipboard(selected);
deleteSelection();

// Paste
const text = await readClipboard();
insertText(text);
```

### Context Menu

```typescript
import { showContextMenu } from "runtime:window";

document.addEventListener("contextmenu", async (e) => {
  e.preventDefault();

  const action = await showContextMenu({
    items: [
      { id: "cut", label: "Cut", accelerator: "Cmd+X" },
      { id: "copy", label: "Copy", accelerator: "Cmd+C" },
      { id: "paste", label: "Paste", accelerator: "Cmd+V" },
      { type: "separator" },
      { id: "selectAll", label: "Select All", accelerator: "Cmd+A" }
    ]
  });

  handleAction(action);
});
```

## File Access Patterns

The capability `read = ["~/*", "./*"]` means:
- `~/*` - Any file in user's home directory
- `./*` - Any file in the app's working directory

For production, consider narrower patterns like:
```toml
read = ["~/Documents/*", "~/Desktop/*"]
write = ["~/Documents/*"]
```
