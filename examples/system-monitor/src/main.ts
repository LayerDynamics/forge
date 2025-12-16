// Forge System Monitor - Main Deno entry point
// Demonstrates: runtime:sys info, runtime:process listing, multi-window, tray icons

import { createWindow, tray, menu, WindowHandle } from "runtime:window";
import { ipcEvents, sendToWindow } from "runtime:ipc";
import { info, powerInfo } from "runtime:sys";
import { spawn } from "runtime:process";

// Note: Process listing would use runtime:process but we'll simulate for demo
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

async function main() {
  console.log("Forge System Monitor starting...");

  // Get system info
  const sysInfo = info();
  const startedAt = Date.now();

  let state: SystemState = {
    hostname: sysInfo.hostname || "Unknown",
    os: sysInfo.os,
    arch: sysInfo.arch,
    cpuCount: sysInfo.cpu_count,
    processes: [],
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
  const trayIcon = await tray.create({
    tooltip: "Forge System Monitor",
    menu: [
      { id: "cpu", label: `CPU: --`, enabled: false },
      { id: "separator", label: "-", type: "separator" },
      { id: "show", label: "Show Monitor" },
      { id: "quit", label: "Quit" }
    ]
  });

  // Open main window
  const mainWindow = await createWindow({
    url: "app://index.html",
    width: 600,
    height: 500,
    title: "Forge System Monitor"
  });

  console.log(`Main window opened with ID: ${mainWindow.id}`);

  // Send state to main window
  function sendState() {
    sendToWindow(mainWindow.id, "state", state);
  }

  async function fetchProcesses(): Promise<ProcessInfo[]> {
    try {
      const proc = await spawn("ps", {
        args: ["-axo", "pid,comm,pcpu,rss,state"],
        stdout: "piped",
      });

      const processes: ProcessInfo[] = [];
      let first = true;
      for await (const line of proc.stdout) {
        const trimmed = line.trim();
        if (!trimmed) continue;
        if (first && trimmed.toLowerCase().startsWith("pid")) {
          first = false;
          continue;
        }
        const match = trimmed.match(/^(\d+)\s+(\S+)\s+([\d.]+)\s+(\d+)\s+(\S+)/);
        if (!match) continue;
        const [, pidStr, name, cpuStr, rssStr, stateChar] = match;
        const pid = Number(pidStr);
        const cpu = Number(cpuStr);
        const rssKb = Number(rssStr);
        const memory = rssKb / 1024; // MB
        const status = stateChar.startsWith("R")
          ? "running"
          : stateChar.startsWith("S")
            ? "sleeping"
            : "other";

        processes.push({ pid, name, cpu, memory, status });
      }

      await proc.wait();
      return processes.sort((a, b) => b.cpu - a.cpu);
    } catch (e) {
      console.warn("Process listing failed:", e);
      return [];
    }
  }

  // Update metrics from real process data
  async function updateMetrics() {
    const processes = await fetchProcesses();
    state.processes = processes;
    state.cpuUsage = processes.reduce((sum, p) => sum + p.cpu, 0);
    state.memoryUsage = processes.reduce((sum, p) => sum + p.memory, 0);
    state.uptime = Math.floor((Date.now() - startedAt) / 1000);

    trayIcon.update({
      tooltip: `CPU: ${state.cpuUsage.toFixed(1)}%`,
      menu: [
        { id: "cpu", label: `CPU: ${state.cpuUsage.toFixed(1)}%`, enabled: false },
        { id: "mem", label: `Memory: ${state.memoryUsage.toFixed(1)} MB`, enabled: false },
        { id: "separator", label: "-", type: "separator" },
        { id: "show", label: "Show Monitor" },
        { id: "quit", label: "Quit" }
      ]
    });

    sendState();
  }

  // Start update interval
  const updateInterval = setInterval(() => {
    updateMetrics().catch((e) => console.error("Update failed:", e));
  }, 2000);
  await updateMetrics();

  // Handle menu events
  menu.onMenu((event) => {
    console.log("Menu event:", event.itemId);

    switch (event.itemId) {
      case "show":
        // Focus main window (would use a show/focus API in production)
        break;
      case "quit":
        clearInterval(updateInterval);
        trayIcon.destroy();
        mainWindow.close();
        for (const win of detailWindows.values()) {
          win.close();
        }
        Deno.exit(0);
        break;
    }
  });

  // Handle window events
  for await (const event of ipcEvents()) {
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
            const detailWin = await createWindow({
              url: `app://detail.html?pid=${pid}`,
              width: 400,
              height: 300,
              title: `Process: ${process.name} (${pid})`
            });
            detailWindows.set(pid, detailWin);

            // Send initial detail data
            setTimeout(() => {
              sendToWindow(detailWin.id, "process-detail", process);
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
                sendToWindow(win.id, "process-detail", process);
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
