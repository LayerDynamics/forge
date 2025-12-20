// Forge Text Editor - Main Deno entry point
// Demonstrates: Full runtime:fs, dialogs, context menus, clipboard, file watching

import { createWindow, menu, dialog } from "runtime:window";
import { windowEvents, sendToWindow } from "runtime:ipc";
import { readTextFile, writeTextFile, watch, exists } from "runtime:fs";
import { clipboard, notify } from "runtime:sys";

interface EditorState {
  filePath: string | null;
  content: string;
  savedContent: string;
  modified: boolean;
  recentFiles: string[];
}

const MAX_RECENT_FILES = 5;

async function main() {
  console.log("Forge Text Editor starting...");

  let state: EditorState = {
    filePath: null,
    content: "",
    savedContent: "",
    modified: false,
    recentFiles: [],
  };

  let fileWatcher: { close(): Promise<void> } | null = null;

  // Build recent files menu
  function buildRecentFilesMenu() {
    if (state.recentFiles.length === 0) {
      return [{ id: "no-recent", label: "No Recent Files", enabled: false }];
    }
    return state.recentFiles.map((path, index) => ({
      id: `recent-${index}`,
      label: path.split("/").pop() || path,
    }));
  }

  // Set up application menu
  async function updateMenu() {
    await menu.setAppMenu([
      {
        label: "File",
        submenu: [
          { id: "new", label: "New", accelerator: "CmdOrCtrl+N" },
          { id: "open", label: "Open...", accelerator: "CmdOrCtrl+O" },
          {
            label: "Open Recent",
            submenu: buildRecentFilesMenu(),
          },
          { id: "separator1", label: "-", type: "separator" },
          { id: "save", label: "Save", accelerator: "CmdOrCtrl+S" },
          { id: "save-as", label: "Save As...", accelerator: "CmdOrCtrl+Shift+S" },
          { id: "separator2", label: "-", type: "separator" },
          { id: "close", label: "Close", accelerator: "CmdOrCtrl+W" },
        ],
      },
      {
        label: "Edit",
        submenu: [
          { id: "undo", label: "Undo", accelerator: "CmdOrCtrl+Z" },
          { id: "redo", label: "Redo", accelerator: "CmdOrCtrl+Shift+Z" },
          { id: "separator3", label: "-", type: "separator" },
          { id: "cut", label: "Cut", accelerator: "CmdOrCtrl+X" },
          { id: "copy", label: "Copy", accelerator: "CmdOrCtrl+C" },
          { id: "paste", label: "Paste", accelerator: "CmdOrCtrl+V" },
          { id: "select-all", label: "Select All", accelerator: "CmdOrCtrl+A" },
          { id: "separator4", label: "-", type: "separator" },
          { id: "find", label: "Find...", accelerator: "CmdOrCtrl+F" },
          { id: "replace", label: "Replace...", accelerator: "CmdOrCtrl+H" },
        ],
      },
      {
        label: "View",
        submenu: [
          { id: "word-wrap", label: "Word Wrap", type: "checkbox", checked: true },
          { id: "line-numbers", label: "Line Numbers", type: "checkbox", checked: true },
        ],
      },
    ]);
  }

  await updateMenu();

  // Open main window
  const win = await createWindow({
    url: "app://index.html",
    width: 800,
    height: 600,
    title: "Forge Editor - Untitled",
  });

  // Update window title
  function updateTitle() {
    const fileName = state.filePath ? state.filePath.split("/").pop() : "Untitled";
    const modified = state.modified ? " *" : "";
    win.setTitle(`Forge Editor - ${fileName}${modified}`);
  }

  // Send state to renderer
  function sendState() {
    sendToWindow(win.id, "state", {
      content: state.content,
      filePath: state.filePath,
      modified: state.modified,
      fileName: state.filePath ? state.filePath.split("/").pop() : "Untitled",
    });
  }

  // Add to recent files
  function addToRecentFiles(path: string) {
    state.recentFiles = [
      path,
      ...state.recentFiles.filter((p) => p !== path),
    ].slice(0, MAX_RECENT_FILES);
    updateMenu();
  }

  // Start watching a file
  async function startWatching(path: string) {
    if (fileWatcher) {
      await fileWatcher.close();
      fileWatcher = null;
    }

    try {
      fileWatcher = await watch(path);
      // Watch for changes in background
      (async () => {
        try {
          while (true) {
            const event = await (fileWatcher as { next(): Promise<{ kind: string } | null> }).next();
            if (!event) break;
            if (event.kind === "modify" && state.filePath === path) {
              // File changed externally
              const result = await dialog.message({
                title: "File Changed",
                message: "The file has been modified externally. Reload?",
                kind: "warning",
                buttons: ["Cancel", "Reload"],
              });
              if (result === 1) {
                const content = await readTextFile(path);
                state.content = content;
                state.savedContent = content;
                state.modified = false;
                sendState();
                updateTitle();
              }
            }
          }
        } catch (e) {
          console.warn("File watcher error:", e);
        }
      })();
    } catch (e) {
      console.warn("Could not start file watcher:", e);
    }
  }

  // Open a file
  async function openFile(path: string) {
    try {
      const content = await readTextFile(path);
      state.filePath = path;
      state.content = content;
      state.savedContent = content;
      state.modified = false;
      addToRecentFiles(path);
      await startWatching(path);
      sendState();
      updateTitle();
    } catch (e) {
      await dialog.error(`Failed to open file: ${e}`);
    }
  }

  // Save current file
  async function saveFile(path?: string) {
    const savePath = path || state.filePath;
    if (!savePath) {
      return saveFileAs();
    }

    try {
      await writeTextFile(savePath, state.content);
      state.filePath = savePath;
      state.savedContent = state.content;
      state.modified = false;
      addToRecentFiles(savePath);
      if (!fileWatcher) {
        await startWatching(savePath);
      }
      sendState();
      updateTitle();
      return true;
    } catch (e) {
      await dialog.error(`Failed to save file: ${e}`);
      return false;
    }
  }

  // Save file as
  async function saveFileAs() {
    const path = await dialog.save({
      title: "Save As",
      filters: [
        { name: "Text Files", extensions: ["txt", "md", "json", "js", "ts"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (path) {
      return saveFile(path);
    }
    return false;
  }

  // Check for unsaved changes
  async function checkUnsaved(): Promise<boolean> {
    if (!state.modified) return true;

    const result = await dialog.message({
      title: "Unsaved Changes",
      message: "Do you want to save your changes?",
      kind: "warning",
      buttons: ["Don't Save", "Cancel", "Save"],
    });

    if (result === 2) {
      // Save
      return await saveFile();
    }
    return result === 0; // Don't Save
  }

  // New file
  async function newFile() {
    if (!(await checkUnsaved())) return;

    if (fileWatcher) {
      await fileWatcher.close();
      fileWatcher = null;
    }

    state.filePath = null;
    state.content = "";
    state.savedContent = "";
    state.modified = false;
    sendState();
    updateTitle();
  }

  // Handle menu events
  menu.onMenu(async (event) => {
    console.log("Menu event:", event.itemId);

    // Handle recent files
    if (event.itemId.startsWith("recent-")) {
      const index = parseInt(event.itemId.replace("recent-", ""));
      const path = state.recentFiles[index];
      if (path && (await checkUnsaved())) {
        await openFile(path);
      }
      return;
    }

    switch (event.itemId) {
      case "new":
        await newFile();
        break;

      case "open": {
        if (!(await checkUnsaved())) break;
        const paths = await dialog.open({
          title: "Open File",
          filters: [
            { name: "Text Files", extensions: ["txt", "md", "json", "js", "ts"] },
            { name: "All Files", extensions: ["*"] },
          ],
        });
        if (paths && paths.length > 0) {
          await openFile(paths[0]);
        }
        break;
      }

      case "save":
        await saveFile();
        break;

      case "save-as":
        await saveFileAs();
        break;

      case "close":
        if (await checkUnsaved()) {
          win.close();
        }
        break;

      case "cut":
      case "copy":
      case "paste":
      case "select-all":
      case "undo":
      case "redo":
      case "find":
      case "replace":
        sendToWindow(win.id, "editor-command", event.itemId);
        break;

      case "word-wrap":
      case "line-numbers":
        sendToWindow(win.id, "toggle-option", event.itemId);
        break;
    }
  });

  // Handle window events
  for await (const event of windowEvents()) {
    console.log("Window event:", event.channel);

    switch (event.channel) {
      case "ready":
        sendState();
        break;

      case "content-changed": {
        state.content = event.payload as string;
        state.modified = state.content !== state.savedContent;
        updateTitle();
        break;
      }

      case "context-menu": {
        const result = await menu.showContextMenu([
          { id: "cut", label: "Cut", accelerator: "CmdOrCtrl+X" },
          { id: "copy", label: "Copy", accelerator: "CmdOrCtrl+C" },
          { id: "paste", label: "Paste", accelerator: "CmdOrCtrl+V" },
          { id: "separator", label: "-", type: "separator" },
          { id: "select-all", label: "Select All", accelerator: "CmdOrCtrl+A" },
        ], win.id);
        if (result) {
          sendToWindow(win.id, "editor-command", result);
        }
        break;
      }

      case "request-paste": {
        const text = await clipboard.read();
        sendToWindow(win.id, "paste-content", text);
        break;
      }

      case "copy-to-clipboard": {
        await clipboard.write(event.payload as string);
        break;
      }

      case "drop-file": {
        const path = event.payload as string;
        if (await checkUnsaved()) {
          await openFile(path);
        }
        break;
      }
    }
  }
}

main().catch(console.error);
