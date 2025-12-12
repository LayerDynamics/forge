import { openWindow, windowEvents } from "host:ui";

console.log("Starting Forge Vue app...");

const win = await openWindow({
  url: "app://index.html",
  width: 1024,
  height: 768,
  title: "Forge Vue App"
});

// Listen for events from the renderer
for await (const event of windowEvents()) {
  console.log("Event from renderer:", event);
}
