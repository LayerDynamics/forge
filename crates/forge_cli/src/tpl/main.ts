import { openWindow } from "runtime:ui";
import { readTextFile } from "runtime:fs";

console.log("Deno main.ts - starting");
await openWindow({ url: "app://index.html", width: 1024, height: 640 });
try {
  const txt = await readTextFile("./README.md"); // just to demo
  console.log("readTextFile README.md length:", txt.length);
} catch (e) {
  console.warn("readTextFile failed (expected in dev):", e);
}
