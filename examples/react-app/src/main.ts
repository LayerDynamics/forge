import { createWindow } from "runtime:window";
import { sendToWindow, windowEvents } from "runtime:ipc";

console.log("Starting React App...");
console.log("createWindow type", typeof createWindow);
// Debug op availability
const coreOps =
    (globalThis as { Deno?: { core?: { ops?: Record<string, unknown> } } }).Deno?.core?.ops;
console.log("op_window_create present?", !!coreOps?.op_window_create);
console.log("createWindow impl", createWindow.toString());

const win = await createWindow({
  url: "app://index.html",
  width: 1024,
  height: 768,
  title: "React App"
});
console.log("Window created:", win.id, "visible?", await win.isVisible());
await win.focus();

// Example: Handle IPC events from the renderer
for await (const event of windowEvents()) {
  console.log("Event from renderer:", event);

  // Echo messages back to demonstrate bidirectional IPC
  if (event.channel === "ping") {
    sendToWindow(win.id, "pong", { timestamp: Date.now() });
  }
}
