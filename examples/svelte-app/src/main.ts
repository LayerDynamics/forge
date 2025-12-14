import { openWindow, windowEvents, sendToWindow } from "host:ui";

console.log("Starting Svelte App...");

const win = await openWindow({
  url: "app://index.html",
  width: 1024,
  height: 768,
  title: "Svelte App"
});

// Handle IPC events from the renderer
for await (const event of windowEvents()) {
  console.log("Event from renderer:", event);

  if (event.channel === "get-todos") {
    // Simulate fetching todos from a backend
    const todos = [
      { id: 1, text: "Learn Svelte", done: true },
      { id: 2, text: "Build a Forge app", done: false },
      { id: 3, text: "Ship it!", done: false }
    ];
    sendToWindow(win.id, "todos-loaded", { todos });
  }

  if (event.channel === "save-todos") {
    const { todos } = event.payload as { todos: unknown[] };
    console.log("Saving todos:", todos);
    // In a real app, you would persist these to disk or a database
    sendToWindow(win.id, "todos-saved", { success: true });
  }
}
