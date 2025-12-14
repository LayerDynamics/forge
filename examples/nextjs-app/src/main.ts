import { openWindow, windowEvents, sendToWindow } from "host:ui";

console.log("Starting Next.js-style App...");

const win = await openWindow({
  url: "app://index.html",
  width: 1024,
  height: 768,
  title: "Next.js App"
});

// Handle navigation and data fetching requests from renderer
for await (const event of windowEvents()) {
  console.log("Event from renderer:", event);

  if (event.channel === "fetch-data") {
    const { page } = event.payload as { page: string };

    // Simulate server-side data fetching
    const data = await fetchPageData(page);
    sendToWindow(win.id, "page-data", { page, data });
  }
}

// Simulate data fetching based on route
async function fetchPageData(page: string): Promise<unknown> {
  // Simulate async data fetch
  await new Promise(resolve => setTimeout(resolve, 100));

  switch (page) {
    case "/":
      return { title: "Home", content: "Welcome to the Next.js-style Forge app!" };
    case "/about":
      return { title: "About", content: "This demonstrates Next.js patterns in Forge." };
    case "/dashboard":
      return {
        title: "Dashboard",
        stats: [
          { label: "Users", value: 1234 },
          { label: "Revenue", value: "$12,345" },
          { label: "Orders", value: 567 }
        ]
      };
    default:
      return { title: "404", content: "Page not found" };
  }
}
