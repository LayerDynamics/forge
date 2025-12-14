import { openWindow, windowEvents, sendToWindow } from "host:ui";

console.log("Starting React App...");

const win = await openWindow({
  url: "app://index.html",
  width: 1024,
  height: 768,
  title: "React App"
});

// Example: Handle IPC events from the renderer
for await (const event of windowEvents()) {
  console.log("Event from renderer:", event);

  // Echo messages back to demonstrate bidirectional IPC
  if (event.channel === "ping") {
    sendToWindow(win.id, "pong", { timestamp: Date.now() });
  }
}
