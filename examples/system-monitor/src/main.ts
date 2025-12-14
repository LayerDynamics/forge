// Forge System Monitor - Main Deno entry point
// Demonstrates: host:sys info, host:process listing, multi-window, tray icons

import { openWindow, createTray, onMenu, windowEvents, WindowHandle } from "host:ui";
import { info, powerInfo } from "host:sys";

// Note: Process listing would use host:process but we'll simulate for demo
// In a full implementation, this would use `spawn("ps", { args: ["aux"] })`
// and parse the output, or use a native process listing op

interface ProcessInfo {
  pid: number;
  name: string;
  cpu: number;
  memory: number;
  status: string;
}

interface SystemState {
  hostname: string;
  os: string;
  arch: string;
  cpuCount: number;
  processes: ProcessInfo[];
  cpuUsage: number;
  memoryUsage: number;
  uptime: number;
  battery: {
    hasBattery: boolean;
    percent: number;
    charging: boolean;
  } | null;
}

// Simulated process data (in real app, this would come from host:process)
function generateMockProcesses(): ProcessInfo[] {
  const names = [
    "kernel_task", "WindowServer", "Finder", "Dock", "SystemUIServer",
    "loginwindow", "forge-host", "Spotlight", "mds_stores", "launchd",
    "configd", "distnoted", "UserEventAgent", "coreaudiod", "bluetoothd"
  ];

  return names.map((name, i) => ({
    pid: 100 + i * 50 + Math.floor(Math.random() * 50),
    name,
    cpu: Math.random() * 10,
    memory: Math.random() * 500 + 50,
    status: Math.random() > 0.1 ? "running" : "sleeping"
  })).sort((a, b) => b.cpu - a.cpu);
}

async function main() {
  console.log("Forge System Monitor starting...");

  // Get system info
  const sysInfo = info();

  let state: SystemState = {
    hostname: sysInfo.hostname || "Unknown",
    os: sysInfo.os,
    arch: sysInfo.arch,
    cpuCount: sysInfo.cpu_count,
    processes: generateMockProcesses(),
    cpuUsage: 0,
    memoryUsage: 0,
    uptime: 0,
    battery: null
  };

  // Try to get battery info
  try {
    const power = await powerInfo();
    if (power.has_battery && power.batteries.length > 0) {
      state.battery = {
        hasBattery: true,
        percent: power.batteries[0].charge_percent,
        charging: power.ac_connected
      };
    }
  } catch (e) {
    console.warn("Could not get battery info:", e);
  }

  // Detail windows map
  const detailWindows = new Map<number, WindowHandle>();

  // Create tray icon
  const tray = await createTray({
    tooltip: "Forge System Monitor",
    menu: [
      { id: "cpu", label: `CPU: --`, enabled: false },
      { id: "separator", label: "-", type: "separator" },
      { id: "show", label: "Show Monitor" },
      { id: "quit", label: "Quit" }
    ]
  });

  // Open main window
  const mainWindow = await openWindow({
    url: "app://index.html",
    width: 600,
    height: 500,
    title: "Forge System Monitor"
  });

  console.log(`Main window opened with ID: ${mainWindow.id}`);

  // Send state to main window
  function sendState() {
    mainWindow.send("state", state);
  }

  // Update simulated metrics
  function updateMetrics() {
    // Simulate CPU usage (would be real in production)
    state.cpuUsage = 10 + Math.random() * 30;
    state.memoryUsage = 40 + Math.random() * 20;
    state.uptime = Math.floor(Date.now() / 1000) % 86400;

    // Update processes with slight variations
    state.processes = state.processes.map(p => ({
      ...p,
      cpu: Math.max(0, p.cpu + (Math.random() - 0.5) * 2),
      memory: Math.max(50, p.memory + (Math.random() - 0.5) * 20)
    })).sort((a, b) => b.cpu - a.cpu);

    // Update tray
    tray.update({
      tooltip: `CPU: ${state.cpuUsage.toFixed(1)}%`,
      menu: [
        { id: "cpu", label: `CPU: ${state.cpuUsage.toFixed(1)}%`, enabled: false },
        { id: "mem", label: `Memory: ${state.memoryUsage.toFixed(1)}%`, enabled: false },
        { id: "separator", label: "-", type: "separator" },
        { id: "show", label: "Show Monitor" },
        { id: "quit", label: "Quit" }
      ]
    });

    sendState();
  }

  // Start update interval
  const updateInterval = setInterval(updateMetrics, 2000);
  updateMetrics();

  // Handle menu events
  onMenu((event) => {
    console.log("Menu event:", event.itemId);

    switch (event.itemId) {
      case "show":
        // Focus main window (would use a show/focus API in production)
        break;
      case "quit":
        clearInterval(updateInterval);
        tray.destroy();
        mainWindow.close();
        for (const win of detailWindows.values()) {
          win.close();
        }
        Deno.exit(0);
        break;
    }
  });

  // Handle window events
  for await (const event of windowEvents()) {
    console.log("Window event:", event.channel, event.windowId);

    // Route events based on window
    if (event.windowId === mainWindow.id) {
      switch (event.channel) {
        case "ready":
          sendState();
          break;

        case "open-process-detail": {
          const pid = event.payload as number;
          const process = state.processes.find(p => p.pid === pid);
          if (process && !detailWindows.has(pid)) {
            const detailWin = await openWindow({
              url: `app://detail.html?pid=${pid}`,
              width: 400,
              height: 300,
              title: `Process: ${process.name} (${pid})`
            });
            detailWindows.set(pid, detailWin);

            // Send initial detail data
            setTimeout(() => {
              detailWin.send("process-detail", process);
            }, 100);
          }
          break;
        }

        case "refresh":
          updateMetrics();
          break;
      }
    } else {
      // Detail window event
      switch (event.channel) {
        case "ready": {
          // Find which detail window this is
          for (const [pid, win] of detailWindows) {
            if (win.id === event.windowId) {
              const process = state.processes.find(p => p.pid === pid);
              if (process) {
                win.send("process-detail", process);
              }
              break;
            }
          }
          break;
        }

        case "close-detail": {
          for (const [pid, win] of detailWindows) {
            if (win.id === event.windowId) {
              await win.close();
              detailWindows.delete(pid);
              break;
            }
          }
          break;
        }
      }
    }
  }
}

main().catch(console.error);
