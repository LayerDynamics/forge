import { createWindow } from "runtime:window";

console.log("ExampleDenoApp main.ts booting...");
await createWindow({ url: "app://index.html", width: 960, height: 600 });
