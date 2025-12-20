// Forge Todo App - Main Deno entry point
// Demonstrates: runtime:fs persistence, runtime:window windows/menus/dialogs, IPC patterns

import { createWindow, menu, dialog } from "runtime:window";
import { windowEvents, sendToWindow } from "runtime:ipc";
import { readTextFile, writeTextFile, exists } from "runtime:fs";
import { homeDir } from "runtime:sys";

// Types
interface Todo {
  id: string;
  text: string;
  completed: boolean;
  createdAt: number;
}

interface TodoState {
  todos: Todo[];
  filter: "all" | "active" | "completed";
}

// Get the path for storing todos
function getTodosPath(): string {
  const home = homeDir();
  if (!home) {
    throw new Error("Could not determine home directory");
  }
  return `${home}/.forge-todo.json`;
}

// Load todos from disk
async function loadTodos(): Promise<Todo[]> {
  const path = getTodosPath();
  try {
    if (await exists(path)) {
      const content = await readTextFile(path);
      const data = JSON.parse(content);
      return data.todos || [];
    }
  } catch (e) {
    console.warn("Failed to load todos:", e);
  }
  return [];
}

// Save todos to disk
async function saveTodos(todos: Todo[]): Promise<void> {
  const path = getTodosPath();
  const content = JSON.stringify({ todos, savedAt: Date.now() }, null, 2);
  await writeTextFile(path, content);
  console.log(`Saved ${todos.length} todos to ${path}`);
}

// Generate unique ID
function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

// Main application
async function main() {
  console.log("Forge Todo App starting...");

  // Load initial todos
  let state: TodoState = {
    todos: await loadTodos(),
    filter: "all"
  };

  // Set up application menu
  await menu.setAppMenu([
    {
      label: "File",
      submenu: [
        { id: "new-todo", label: "New Todo", accelerator: "CmdOrCtrl+N" },
        { id: "separator", label: "-", type: "separator" },
        { id: "clear-completed", label: "Clear Completed" },
        { id: "separator", label: "-", type: "separator" },
        { id: "export", label: "Export...", accelerator: "CmdOrCtrl+E" }
      ]
    },
    {
      label: "View",
      submenu: [
        { id: "filter-all", label: "All", accelerator: "CmdOrCtrl+1" },
        { id: "filter-active", label: "Active", accelerator: "CmdOrCtrl+2" },
        { id: "filter-completed", label: "Completed", accelerator: "CmdOrCtrl+3" }
      ]
    }
  ]);

  // Open main window
  const win = await createWindow({
    url: "app://index.html",
    width: 480,
    height: 640,
    title: "Forge Todo"
  });

  console.log(`Window opened with ID: ${win.id}`);

  // Send initial state to renderer
  function sendState() {
    const filteredTodos = state.todos.filter(todo => {
      if (state.filter === "active") return !todo.completed;
      if (state.filter === "completed") return todo.completed;
      return true;
    });

    sendToWindow(win.id, "state", {
      todos: filteredTodos,
      allTodos: state.todos,
      filter: state.filter,
      counts: {
        all: state.todos.length,
        active: state.todos.filter(t => !t.completed).length,
        completed: state.todos.filter(t => t.completed).length
      }
    });
  }

  // Handle menu events
  menu.onMenu(async (event) => {
    console.log("Menu event:", event);

    switch (event.itemId) {
      case "new-todo":
        sendToWindow(win.id, "focus-input");
        break;
      case "clear-completed":
        state.todos = state.todos.filter(t => !t.completed);
        await saveTodos(state.todos);
        sendState();
        break;
      case "filter-all":
        state.filter = "all";
        sendState();
        break;
      case "filter-active":
        state.filter = "active";
        sendState();
        break;
      case "filter-completed":
        state.filter = "completed";
        sendState();
        break;
      case "export":
        const savePath = await dialog.save({
          title: "Export Todos",
          filters: [{ name: "JSON", extensions: ["json"] }]
        });
        if (savePath) {
          await writeTextFile(savePath, JSON.stringify(state.todos, null, 2));
          await dialog.alert(`Exported ${state.todos.length} todos to ${savePath}`);
        }
        break;
    }
  });

  // Handle window/IPC events
  for await (const event of windowEvents()) {
    console.log("Window event:", event.channel, event.payload);

    switch (event.channel) {
      case "ready":
        // Renderer is ready, send initial state
        sendState();
        break;

      case "add-todo": {
        const text = event.payload as string;
        if (text && text.trim()) {
          const newTodo: Todo = {
            id: generateId(),
            text: text.trim(),
            completed: false,
            createdAt: Date.now()
          };
          state.todos.push(newTodo);
          await saveTodos(state.todos);
          sendState();
        }
        break;
      }

      case "toggle-todo": {
        const id = event.payload as string;
        const todo = state.todos.find(t => t.id === id);
        if (todo) {
          todo.completed = !todo.completed;
          await saveTodos(state.todos);
          sendState();
        }
        break;
      }

      case "delete-todo": {
        const id = event.payload as string;
        state.todos = state.todos.filter(t => t.id !== id);
        await saveTodos(state.todos);
        sendState();
        break;
      }

      case "edit-todo": {
        const { id, text } = event.payload as { id: string; text: string };
        const todo = state.todos.find(t => t.id === id);
        if (todo && text.trim()) {
          todo.text = text.trim();
          await saveTodos(state.todos);
          sendState();
        }
        break;
      }

      case "set-filter": {
        state.filter = event.payload as "all" | "active" | "completed";
        sendState();
        break;
      }

      case "clear-completed": {
        state.todos = state.todos.filter(t => !t.completed);
        await saveTodos(state.todos);
        sendState();
        break;
      }
    }
  }
}

main().catch(console.error);
