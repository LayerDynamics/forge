import { openWindow } from "host:ui";

console.log("ExampleDenoApp main.ts booting...");
await openWindow({ url: "app://index.html", width: 960, height: 600 });
