// Developer Toolkit - Main Deno entry point
// Demonstrates: codesign, crypto, fs, process, shell, sys, storage, window, ipc
//
// This example showcases advanced Forge extension capabilities in a practical
// developer-focused application.

import { createWindow, tray, menu, dialog, WindowHandle } from "runtime:window";
import { windowEvents, sendToWindow } from "runtime:ipc";
import { info, clipboard, notify } from "runtime:sys";
import { readFile, writeFile, stat, readDir } from "runtime:fs";
import { spawn } from "runtime:process";
import { hash, randomBytes } from "runtime:crypto";
import { get, set } from "runtime:storage";
import {
  sign,
  signAdhoc,
  verify,
  listIdentities,
  getIdentityInfo,
  checkCapabilities,
  getEntitlements,
} from "runtime:codesign";

// ============================================================================
// Types
// ============================================================================

interface SigningIdentityInfo {
  id: string;
  name: string;
  expires: string | null;
  valid: boolean;
  type: string;
}

interface FileHashResult {
  path: string;
  name: string;
  size: number;
  hashes: {
    md5: string;
    sha1: string;
    sha256: string;
    sha512: string;
  };
}

interface SigningResult {
  success: boolean;
  path: string;
  identity?: string;
  message: string;
  timestamp?: string;
}

interface VerifyResultInfo {
  path: string;
  valid: boolean;
  signer: string | null;
  timestamp: string | null;
  message: string;
  entitlements?: string;
}

interface AppState {
  // System info
  system: {
    hostname: string;
    os: string;
    arch: string;
    cpuCount: number;
  };
  // Codesign capabilities
  capabilities: {
    codesign: boolean;
    security: boolean;
    signtool: boolean;
    certutil: boolean;
    platform: string;
  };
  // Available signing identities
  identities: SigningIdentityInfo[];
  // Recent operations
  recentHashes: FileHashResult[];
  recentSignings: SigningResult[];
  recentVerifications: VerifyResultInfo[];
  // Preferences
  preferences: {
    defaultIdentity: string | null;
    hardenedRuntime: boolean;
    deepSign: boolean;
    timestampUrl: string;
  };
}

// ============================================================================
// State Management
// ============================================================================

let state: AppState = {
  system: {
    hostname: "",
    os: "",
    arch: "",
    cpuCount: 0,
  },
  capabilities: {
    codesign: false,
    security: false,
    signtool: false,
    certutil: false,
    platform: "unknown",
  },
  identities: [],
  recentHashes: [],
  recentSignings: [],
  recentVerifications: [],
  preferences: {
    defaultIdentity: null,
    hardenedRuntime: true,
    deepSign: true,
    timestampUrl: "http://timestamp.digicert.com",
  },
};

let mainWindow: WindowHandle | null = null;

// ============================================================================
// Initialization
// ============================================================================

async function initializeState(): Promise<void> {
  console.log("Initializing Developer Toolkit state...");

  // Get system info
  const sysInfo = info();
  state.system = {
    hostname: sysInfo.hostname || "Unknown",
    os: sysInfo.os,
    arch: sysInfo.arch,
    cpuCount: sysInfo.cpu_count,
  };

  // Get codesign capabilities
  state.capabilities = checkCapabilities();
  console.log("Platform capabilities:", state.capabilities);

  // Load identities if codesigning is available
  if (state.capabilities.codesign || state.capabilities.signtool) {
    try {
      const identities = await listIdentities();
      state.identities = identities.map((id) => ({
        id: id.id,
        name: id.name,
        expires: id.expires,
        valid: id.valid,
        type: id.type,
      }));
      console.log(`Found ${state.identities.length} signing identities`);
    } catch (e) {
      console.warn("Could not list signing identities:", e);
    }
  }

  // Load saved preferences
  try {
    const savedPrefs = await get("preferences");
    if (savedPrefs) {
      state.preferences = { ...state.preferences, ...JSON.parse(savedPrefs) };
    }
  } catch (e) {
    console.warn("Could not load preferences:", e);
  }

  // Load recent operations
  try {
    const recentHashes = await get("recentHashes");
    if (recentHashes) {
      state.recentHashes = JSON.parse(recentHashes);
    }
  } catch (e) {
    console.warn("Could not load recent hashes:", e);
  }
}

// ============================================================================
// File Hashing Operations
// ============================================================================

async function hashFile(path: string): Promise<FileHashResult> {
  console.log(`Hashing file: ${path}`);

  // Read file
  const content = await readFile(path);
  const fileInfo = await stat(path);

  // Compute all hashes using runtime:crypto
  const [md5Hash, sha1Hash, sha256Hash, sha512Hash] = await Promise.all([
    hash("md5", content),
    hash("sha1", content),
    hash("sha256", content),
    hash("sha512", content),
  ]);

  const result: FileHashResult = {
    path,
    name: path.split("/").pop() || path,
    size: fileInfo.size,
    hashes: {
      md5: md5Hash,
      sha1: sha1Hash,
      sha256: sha256Hash,
      sha512: sha512Hash,
    },
  };

  // Store in recent
  state.recentHashes.unshift(result);
  if (state.recentHashes.length > 10) {
    state.recentHashes = state.recentHashes.slice(0, 10);
  }

  // Persist
  await set("recentHashes", JSON.stringify(state.recentHashes));

  return result;
}

async function hashMultipleFiles(paths: string[]): Promise<FileHashResult[]> {
  const results: FileHashResult[] = [];
  for (const path of paths) {
    try {
      const result = await hashFile(path);
      results.push(result);
    } catch (e) {
      console.error(`Failed to hash ${path}:`, e);
    }
  }
  return results;
}

// ============================================================================
// Code Signing Operations
// ============================================================================

async function signFile(
  path: string,
  identity: string,
  options?: {
    entitlements?: string;
    hardenedRuntime?: boolean;
    deep?: boolean;
  }
): Promise<SigningResult> {
  console.log(`Signing file: ${path} with identity: ${identity}`);

  try {
    await sign({
      path,
      identity,
      entitlements: options?.entitlements,
      hardenedRuntime: options?.hardenedRuntime ?? state.preferences.hardenedRuntime,
      deep: options?.deep ?? state.preferences.deepSign,
      timestampUrl: state.preferences.timestampUrl,
    });

    const result: SigningResult = {
      success: true,
      path,
      identity,
      message: "Successfully signed",
      timestamp: new Date().toISOString(),
    };

    // Store in recent
    state.recentSignings.unshift(result);
    if (state.recentSignings.length > 10) {
      state.recentSignings = state.recentSignings.slice(0, 10);
    }

    // Show notification
    await notify({
      title: "Signing Complete",
      body: `Successfully signed ${path.split("/").pop()}`,
    });

    return result;
  } catch (e) {
    const result: SigningResult = {
      success: false,
      path,
      identity,
      message: e instanceof Error ? e.message : String(e),
    };

    state.recentSignings.unshift(result);
    return result;
  }
}

async function signAdhocFile(path: string): Promise<SigningResult> {
  console.log(`Ad-hoc signing file: ${path}`);

  try {
    await signAdhoc(path);

    const result: SigningResult = {
      success: true,
      path,
      identity: "ad-hoc",
      message: "Successfully ad-hoc signed",
      timestamp: new Date().toISOString(),
    };

    state.recentSignings.unshift(result);
    return result;
  } catch (e) {
    return {
      success: false,
      path,
      identity: "ad-hoc",
      message: e instanceof Error ? e.message : String(e),
    };
  }
}

async function verifyFile(path: string): Promise<VerifyResultInfo> {
  console.log(`Verifying signature: ${path}`);

  const verifyResult = await verify(path);

  // Try to get entitlements (macOS only)
  let entitlements: string | undefined;
  if (state.capabilities.codesign) {
    try {
      entitlements = await getEntitlements(path);
    } catch (e) {
      // Ignore - might not have entitlements
    }
  }

  const result: VerifyResultInfo = {
    path,
    valid: verifyResult.valid,
    signer: verifyResult.signer,
    timestamp: verifyResult.timestamp,
    message: verifyResult.message,
    entitlements,
  };

  state.recentVerifications.unshift(result);
  if (state.recentVerifications.length > 10) {
    state.recentVerifications = state.recentVerifications.slice(0, 10);
  }

  return result;
}

// ============================================================================
// Utility Operations
// ============================================================================

async function copyToClipboard(text: string): Promise<void> {
  await clipboard.write(text);
  await notify({
    title: "Copied",
    body: "Content copied to clipboard",
  });
}

async function savePreferences(): Promise<void> {
  await set("preferences", JSON.stringify(state.preferences));
}

async function generateRandomString(length: number): Promise<string> {
  const bytes = await randomBytes(Math.ceil(length / 2));
  return bytes.slice(0, length);
}

// ============================================================================
// File Operations (using runtime:process for advanced inspection)
// ============================================================================

async function inspectBinary(path: string): Promise<Record<string, string>> {
  const result: Record<string, string> = {};

  // Use 'file' command to get file type
  try {
    const fileProc = await spawn("file", {
      args: ["-b", path],
      stdout: "piped",
    });
    let fileType = "";
    for await (const line of fileProc.stdout) {
      fileType += line;
    }
    await fileProc.wait();
    result.type = fileType.trim();
  } catch (e) {
    result.type = "Unknown";
  }

  // On macOS, use otool to get architecture info
  if (state.system.os === "macos") {
    try {
      const otoolProc = await spawn("lipo", {
        args: ["-archs", path],
        stdout: "piped",
      });
      let archs = "";
      for await (const line of otoolProc.stdout) {
        archs += line;
      }
      await otoolProc.wait();
      result.architectures = archs.trim();
    } catch (e) {
      // Might not be a binary
    }
  }

  return result;
}

// ============================================================================
// Window & UI Management
// ============================================================================

function sendState(): void {
  if (mainWindow) {
    sendToWindow(mainWindow.id, "state", state);
  }
}

async function openFileDialog(
  title: string,
  filters?: { name: string; extensions: string[] }[]
): Promise<string | null> {
  const result = await dialog.open({
    title,
    multiple: false,
    filters,
  });

  if (result && !Array.isArray(result)) {
    return result;
  }
  return null;
}

async function openMultiFileDialog(
  title: string,
  filters?: { name: string; extensions: string[] }[]
): Promise<string[]> {
  const result = await dialog.open({
    title,
    multiple: true,
    filters,
  });

  if (result) {
    return Array.isArray(result) ? result : [result];
  }
  return [];
}

// ============================================================================
// Main Application
// ============================================================================

async function main(): Promise<void> {
  console.log("Developer Toolkit starting...");

  // Initialize state
  await initializeState();

  // Create tray icon
  const trayIcon = await tray.create({
    tooltip: "Developer Toolkit",
    menu: [
      { id: "show", label: "Show Toolkit" },
      { id: "separator", label: "-", type: "separator" },
      { id: "hash", label: "Hash File..." },
      { id: "verify", label: "Verify Signature..." },
      { id: "separator2", label: "-", type: "separator" },
      { id: "quit", label: "Quit" },
    ],
  });

  // Create main window
  mainWindow = await createWindow({
    url: "app://index.html",
    width: 900,
    height: 700,
    title: "Developer Toolkit",
  });

  console.log(`Main window opened with ID: ${mainWindow.id}`);

  // Handle menu events
  menu.onMenu(async (event) => {
    console.log("Menu event:", event.itemId);

    switch (event.itemId) {
      case "show":
        // Focus would go here
        break;

      case "hash": {
        const file = await openFileDialog("Select file to hash");
        if (file) {
          const result = await hashFile(file);
          sendState();
          if (mainWindow) {
            sendToWindow(mainWindow.id, "hash-result", result);
          }
        }
        break;
      }

      case "verify": {
        const file = await openFileDialog("Select file to verify", [
          { name: "Applications", extensions: ["app", "exe", "dmg"] },
          { name: "All Files", extensions: ["*"] },
        ]);
        if (file) {
          const result = await verifyFile(file);
          sendState();
          if (mainWindow) {
            sendToWindow(mainWindow.id, "verify-result", result);
          }
        }
        break;
      }

      case "quit":
        trayIcon.destroy();
        mainWindow?.close();
        Deno.exit(0);
        break;
    }
  });

  // Handle window events
  for await (const event of windowEvents()) {
    console.log("Window event:", event.channel, event.payload);

    switch (event.channel) {
      case "ready":
        sendState();
        break;

      case "get-state":
        sendState();
        break;

      case "refresh-identities": {
        try {
          const identities = await listIdentities();
          state.identities = identities.map((id) => ({
            id: id.id,
            name: id.name,
            expires: id.expires,
            valid: id.valid,
            type: id.type,
          }));
          sendState();
        } catch (e) {
          console.error("Failed to refresh identities:", e);
        }
        break;
      }

      case "select-file-for-hash": {
        const files = await openMultiFileDialog("Select files to hash");
        if (files.length > 0) {
          const results = await hashMultipleFiles(files);
          sendState();
          if (mainWindow) {
            sendToWindow(mainWindow.id, "hash-results", results);
          }
        }
        break;
      }

      case "hash-file": {
        const path = event.payload as string;
        try {
          const result = await hashFile(path);
          sendState();
          if (mainWindow) {
            sendToWindow(mainWindow.id, "hash-result", result);
          }
        } catch (e) {
          if (mainWindow) {
            sendToWindow(mainWindow.id, "error", {
              operation: "hash",
              message: e instanceof Error ? e.message : String(e),
            });
          }
        }
        break;
      }

      case "select-file-for-sign": {
        const file = await openFileDialog("Select file to sign", [
          { name: "Applications", extensions: ["app", "exe", "dmg", "pkg"] },
          { name: "Binaries", extensions: ["*"] },
        ]);
        if (file && mainWindow) {
          sendToWindow(mainWindow.id, "file-selected-for-sign", file);
        }
        break;
      }

      case "sign-file": {
        const { path, identity, options } = event.payload as {
          path: string;
          identity: string;
          options?: { entitlements?: string; hardenedRuntime?: boolean; deep?: boolean };
        };
        const result = await signFile(path, identity, options);
        sendState();
        if (mainWindow) {
          sendToWindow(mainWindow.id, "sign-result", result);
        }
        break;
      }

      case "sign-adhoc": {
        const path = event.payload as string;
        const result = await signAdhocFile(path);
        sendState();
        if (mainWindow) {
          sendToWindow(mainWindow.id, "sign-result", result);
        }
        break;
      }

      case "select-file-for-verify": {
        const file = await openFileDialog("Select file to verify");
        if (file && mainWindow) {
          const result = await verifyFile(file);
          sendState();
          sendToWindow(mainWindow.id, "verify-result", result);
        }
        break;
      }

      case "verify-file": {
        const path = event.payload as string;
        try {
          const result = await verifyFile(path);
          sendState();
          if (mainWindow) {
            sendToWindow(mainWindow.id, "verify-result", result);
          }
        } catch (e) {
          if (mainWindow) {
            sendToWindow(mainWindow.id, "error", {
              operation: "verify",
              message: e instanceof Error ? e.message : String(e),
            });
          }
        }
        break;
      }

      case "inspect-binary": {
        const path = event.payload as string;
        try {
          const result = await inspectBinary(path);
          if (mainWindow) {
            sendToWindow(mainWindow.id, "inspect-result", { path, ...result });
          }
        } catch (e) {
          if (mainWindow) {
            sendToWindow(mainWindow.id, "error", {
              operation: "inspect",
              message: e instanceof Error ? e.message : String(e),
            });
          }
        }
        break;
      }

      case "copy-to-clipboard": {
        const text = event.payload as string;
        await copyToClipboard(text);
        break;
      }

      case "update-preferences": {
        const prefs = event.payload as Partial<typeof state.preferences>;
        state.preferences = { ...state.preferences, ...prefs };
        await savePreferences();
        sendState();
        break;
      }

      case "generate-random": {
        const length = (event.payload as number) || 32;
        const random = await generateRandomString(length);
        if (mainWindow) {
          sendToWindow(mainWindow.id, "random-result", random);
        }
        break;
      }

      case "get-identity-details": {
        const identityId = event.payload as string;
        try {
          const details = await getIdentityInfo(identityId);
          if (mainWindow) {
            sendToWindow(mainWindow.id, "identity-details", details);
          }
        } catch (e) {
          console.error("Failed to get identity details:", e);
        }
        break;
      }
    }
  }
}

main().catch(console.error);
