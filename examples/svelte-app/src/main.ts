// Secure Vault - Backend (Deno)
// Handles all security-sensitive operations: encryption, storage, session management

import { createWindow, menu, tray, dialog } from "runtime:window";
import { windowEvents, sendToWindow } from "runtime:ipc";
import { get, set, remove } from "runtime:storage";
import { randomBytes, randomUUID, encrypt, decrypt, deriveKey } from "runtime:crypto";
import { clipboard, notify } from "runtime:sys";
import { readBytes, writeBytes } from "runtime:fs";

// ext_log - Structured logging (replaces console.log)
import { infoLog, debug, warn, error, setDefaultWindow } from "runtime:log";

// ext_trace - Performance tracing spans (disabled: u128 not supported in Deno ops)
// import { start as traceStart, end as traceEnd, instant } from "runtime:trace";
// TODO: Fix ext_trace to use u64 instead of u128 for span IDs
function traceStart(_name: string, _attributes?: unknown): bigint { return 0n; }
function traceEnd(_id: bigint, _result?: unknown): void { }

// ============================================================================
// Types
// ============================================================================

interface VaultMeta {
  version: number;
  salt: string;           // Base64-encoded PBKDF2 salt
  verifier: string;       // Base64-encoded encrypted verification token
  iv: string;             // Base64-encoded IV for verifier
  tag: string;            // Base64-encoded auth tag for verifier
  createdAt: string;
}

interface EncryptedData {
  ciphertext: string;
  iv: string;
  tag: string;
}

interface NoteEntry {
  id: string;
  title: string;
  category: Category;
  createdAt: string;
  modifiedAt: string;
  preview: string;
}

interface NotesIndex {
  notes: NoteEntry[];
}

interface NoteContent {
  content: string;
}

interface NoteSummary {
  id: string;
  title: string;
  category: Category;
  createdAt: string;
  modifiedAt: string;
  preview: string;
}

interface Note {
  id: string;
  title: string;
  content: string;
  category: Category;
  createdAt: string;
  modifiedAt: string;
}

type Category = "personal" | "work" | "financial" | "medical" | "passwords" | "other";

// File vault types
interface VaultFile {
  id: string;
  name: string;
  size: number;
  mimeType: string;
  category: Category;
  createdAt: string;
}

interface FilesIndex {
  files: VaultFile[];
}

interface VaultSession {
  unlocked: boolean;
  sessionKey: Uint8Array | null;
  lastActivity: number;
  notesIndex: NotesIndex | null;
  filesIndex: FilesIndex | null;
}

// ============================================================================
// Constants
// ============================================================================

const VAULT_VERSION = 1;
const VERIFIER_TOKEN = "FORGE_VAULT_VERIFIED";
const PBKDF2_ITERATIONS = 100000;
const KEY_LENGTH = 32; // 256 bits for AES-256
const LOCK_TIMEOUT_MS = 5 * 60 * 1000;  // 5 minutes
const WARNING_BEFORE_MS = 30 * 1000;     // 30 seconds warning

// ============================================================================
// Session State (in-memory only - never persisted)
// ============================================================================

let session: VaultSession = {
  unlocked: false,
  sessionKey: null,
  lastActivity: Date.now(),
  notesIndex: null,
  filesIndex: null
};

let mainWindowId: string | null = null;

// ============================================================================
// Base64 Helpers
// ============================================================================

function toBase64(bytes: Uint8Array): string {
  return btoa(String.fromCharCode(...bytes));
}

function fromBase64(str: string): Uint8Array {
  return Uint8Array.from(atob(str), c => c.charCodeAt(0));
}

// ============================================================================
// Encryption Helpers
// ============================================================================

function encryptData(data: string, key: Uint8Array): EncryptedData {
  const plaintext = new TextEncoder().encode(data);
  const result = encrypt("aes-256-gcm", key, plaintext);
  return {
    ciphertext: toBase64(result.ciphertext),
    iv: toBase64(result.iv),
    tag: toBase64(result.tag)
  };
}

function decryptData(encrypted: EncryptedData, key: Uint8Array): string {
  const ciphertext = fromBase64(encrypted.ciphertext);
  const iv = fromBase64(encrypted.iv);
  const tag = fromBase64(encrypted.tag);

  const plaintext = decrypt("aes-256-gcm", key, { ciphertext, iv, tag });
  return new TextDecoder().decode(plaintext);
}

// Binary encryption for files
function encryptBytes(data: Uint8Array, key: Uint8Array): EncryptedData {
  const result = encrypt("aes-256-gcm", key, data);
  return {
    ciphertext: toBase64(result.ciphertext),
    iv: toBase64(result.iv),
    tag: toBase64(result.tag)
  };
}

function decryptBytes(encrypted: EncryptedData, key: Uint8Array): Uint8Array {
  const ciphertext = fromBase64(encrypted.ciphertext);
  const iv = fromBase64(encrypted.iv);
  const tag = fromBase64(encrypted.tag);
  return decrypt("aes-256-gcm", key, { ciphertext, iv, tag });
}

// ============================================================================
// Vault Operations
// ============================================================================

async function isVaultInitialized(): Promise<boolean> {
  const meta = await get<VaultMeta>("vault:meta");
  return meta !== null && meta.version === VAULT_VERSION;
}

async function initializeVault(password: string): Promise<{ success: boolean; error?: string }> {
  const spanId = traceStart("vault:initialize");
  try {
    // Generate salt for PBKDF2
    const salt = randomBytes(16);

    // Derive key from password
    const key = deriveKey(password, salt, PBKDF2_ITERATIONS, KEY_LENGTH);

    // Encrypt verification token
    const verifierEncrypted = encryptData(VERIFIER_TOKEN, key);

    // Store vault metadata
    const meta: VaultMeta = {
      version: VAULT_VERSION,
      salt: toBase64(salt),
      verifier: verifierEncrypted.ciphertext,
      iv: verifierEncrypted.iv,
      tag: verifierEncrypted.tag,
      createdAt: new Date().toISOString()
    };

    await set("vault:meta", meta);

    // Initialize empty notes index
    const emptyNotesIndex: NotesIndex = { notes: [] };
    const encryptedNotesIndex = encryptData(JSON.stringify(emptyNotesIndex), key);
    await set("vault:notes", encryptedNotesIndex);

    // Initialize empty files index
    const emptyFilesIndex: FilesIndex = { files: [] };
    const encryptedFilesIndex = encryptData(JSON.stringify(emptyFilesIndex), key);
    await set("vault:files", encryptedFilesIndex);

    // Set session state
    session.sessionKey = key;
    session.unlocked = true;
    session.lastActivity = Date.now();
    session.notesIndex = emptyNotesIndex;
    session.filesIndex = emptyFilesIndex;

    traceEnd(spanId, { success: true });
    // instant("vault:initialized");  // TODO: ext_trace u128 not supported
    return { success: true };
  } catch (err) {
    traceEnd(spanId, { success: false, error: String(err) });
    return { success: false, error: String(err) };
  }
}

async function unlockVault(password: string): Promise<{ success: boolean; error?: string }> {
  const spanId = traceStart("vault:unlock");
  try {
    const meta = await get<VaultMeta>("vault:meta");
    if (!meta) {
      traceEnd(spanId, { success: false, error: "Vault not initialized" });
      return { success: false, error: "Vault not initialized" };
    }

    // Derive key from password
    const salt = fromBase64(meta.salt);
    const key = deriveKey(password, salt, PBKDF2_ITERATIONS, KEY_LENGTH);

    // Try to decrypt verifier
    try {
      const decrypted = decryptData({
        ciphertext: meta.verifier,
        iv: meta.iv,
        tag: meta.tag
      }, key);

      if (decrypted !== VERIFIER_TOKEN) {
        traceEnd(spanId, { success: false, error: "Invalid password" });
        return { success: false, error: "Invalid password" };
      }
    } catch {
      traceEnd(spanId, { success: false, error: "Invalid password" });
      return { success: false, error: "Invalid password" };
    }

    // Load and decrypt notes index
    const encryptedNotesIndex = await get<EncryptedData>("vault:notes");
    if (encryptedNotesIndex) {
      const indexJson = decryptData(encryptedNotesIndex, key);
      session.notesIndex = JSON.parse(indexJson) as NotesIndex;
    } else {
      session.notesIndex = { notes: [] };
    }

    // Load and decrypt files index
    const encryptedFilesIndex = await get<EncryptedData>("vault:files");
    if (encryptedFilesIndex) {
      const indexJson = decryptData(encryptedFilesIndex, key);
      session.filesIndex = JSON.parse(indexJson) as FilesIndex;
    } else {
      session.filesIndex = { files: [] };
    }

    // Set session state
    session.sessionKey = key;
    session.unlocked = true;
    session.lastActivity = Date.now();

    traceEnd(spanId, { success: true });
    // instant("vault:unlocked");  // TODO: ext_trace u128 not supported
    return { success: true };
  } catch (err) {
    traceEnd(spanId, { success: false, error: String(err) });
    return { success: false, error: String(err) };
  }
}

function lockVault(): void {
  // instant("vault:locked");  // TODO: ext_trace u128 not supported
  // Clear sensitive data from memory
  if (session.sessionKey) {
    session.sessionKey.fill(0);
  }
  session.sessionKey = null;
  session.unlocked = false;
  session.notesIndex = null;
  session.filesIndex = null;
}

function resetActivityTimer(): void {
  session.lastActivity = Date.now();
}

// ============================================================================
// Notes Operations
// ============================================================================

async function saveNotesIndex(): Promise<void> {
  if (!session.sessionKey || !session.notesIndex) return;

  const encryptedIndex = encryptData(JSON.stringify(session.notesIndex), session.sessionKey);
  await set("vault:notes", encryptedIndex);
}

async function createNote(title: string, content: string, category: Category): Promise<Note | null> {
  if (!session.sessionKey || !session.notesIndex) return null;

  const spanId = traceStart("note:create", { title, category });
  const id = randomUUID();
  const now = new Date().toISOString();
  const preview = content.substring(0, 100).replace(/\n/g, " ");

  // Create note entry for index
  const entry: NoteEntry = {
    id,
    title,
    category,
    createdAt: now,
    modifiedAt: now,
    preview
  };

  // Add to index
  session.notesIndex.notes.unshift(entry);
  await saveNotesIndex();

  // Store encrypted content
  const noteContent: NoteContent = { content };
  const encryptedContent = encryptData(JSON.stringify(noteContent), session.sessionKey);
  await set(`note:${id}`, encryptedContent);

  traceEnd(spanId, { success: true, noteId: id });
  return {
    id,
    title,
    content,
    category,
    createdAt: now,
    modifiedAt: now
  };
}

async function getNote(id: string): Promise<Note | null> {
  const spanId = traceStart("note:get", { noteId: id });
  if (!session.sessionKey || !session.notesIndex) {
    traceEnd(spanId, { success: false, error: "session locked" });
    return null;
  }

  const entry = session.notesIndex.notes.find(n => n.id === id);
  if (!entry) {
    traceEnd(spanId, { success: false, error: "note not found" });
    return null;
  }

  const encryptedContent = await get<EncryptedData>(`note:${id}`);
  if (!encryptedContent) {
    traceEnd(spanId, { success: false, error: "no encrypted content" });
    return null;
  }

  try {
    const contentJson = decryptData(encryptedContent, session.sessionKey);
    const noteContent = JSON.parse(contentJson) as NoteContent;

    traceEnd(spanId, { success: true });
    return {
      id: entry.id,
      title: entry.title,
      content: noteContent.content,
      category: entry.category,
      createdAt: entry.createdAt,
      modifiedAt: entry.modifiedAt
    };
  } catch {
    traceEnd(spanId, { success: false, error: "decryption failed" });
    return null;
  }
}

async function updateNote(id: string, updates: { title?: string; content?: string; category?: Category }): Promise<boolean> {
  const spanId = traceStart("note:update", { noteId: id });
  if (!session.sessionKey || !session.notesIndex) {
    traceEnd(spanId, { success: false, error: "session locked" });
    return false;
  }

  const entryIndex = session.notesIndex.notes.findIndex(n => n.id === id);
  if (entryIndex === -1) {
    traceEnd(spanId, { success: false, error: "note not found" });
    return false;
  }

  const entry = session.notesIndex.notes[entryIndex];
  const now = new Date().toISOString();

  // Update entry
  if (updates.title !== undefined) entry.title = updates.title;
  if (updates.category !== undefined) entry.category = updates.category;
  if (updates.content !== undefined) {
    entry.preview = updates.content.substring(0, 100).replace(/\n/g, " ");

    // Update encrypted content
    const noteContent: NoteContent = { content: updates.content };
    const encryptedContent = encryptData(JSON.stringify(noteContent), session.sessionKey);
    await set(`note:${id}`, encryptedContent);
  }

  entry.modifiedAt = now;
  await saveNotesIndex();

  traceEnd(spanId, { success: true });
  return true;
}

async function deleteNote(id: string): Promise<boolean> {
  const spanId = traceStart("note:delete", { noteId: id });
  if (!session.sessionKey || !session.notesIndex) {
    traceEnd(spanId, { success: false, error: "session locked" });
    return false;
  }

  const entryIndex = session.notesIndex.notes.findIndex(n => n.id === id);
  if (entryIndex === -1) {
    traceEnd(spanId, { success: false, error: "note not found" });
    return false;
  }

  // Remove from index
  session.notesIndex.notes.splice(entryIndex, 1);
  await saveNotesIndex();

  // Remove encrypted content
  await remove(`note:${id}`);

  traceEnd(spanId, { success: true });
  return true;
}

interface NoteListItem {
  id: string;
  title: string;
  content: string;  // Use preview as content for list display
  category: Category;
  createdAt: string;
  modifiedAt: string;
}

function getFilteredNotes(category?: string, search?: string): NoteListItem[] {
  if (!session.notesIndex) return [];

  let notes = session.notesIndex.notes;

  // Filter by category
  if (category && category !== "all") {
    notes = notes.filter(n => n.category === category);
  }

  // Filter by search
  if (search) {
    const searchLower = search.toLowerCase();
    notes = notes.filter(n =>
      n.title.toLowerCase().includes(searchLower) ||
      n.preview.toLowerCase().includes(searchLower)
    );
  }

  // Return with content = preview for frontend compatibility
  return notes.map(n => ({
    id: n.id,
    title: n.title,
    content: n.preview,  // Frontend expects 'content' field
    category: n.category,
    createdAt: n.createdAt,
    modifiedAt: n.modifiedAt
  }));
}

// ============================================================================
// File Operations
// ============================================================================

async function saveFilesIndex(): Promise<void> {
  if (!session.sessionKey || !session.filesIndex) return;

  const encryptedIndex = encryptData(JSON.stringify(session.filesIndex), session.sessionKey);
  await set("vault:files", encryptedIndex);
}

function getMimeType(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  const mimeTypes: Record<string, string> = {
    // Images
    'jpg': 'image/jpeg', 'jpeg': 'image/jpeg', 'png': 'image/png',
    'gif': 'image/gif', 'webp': 'image/webp', 'svg': 'image/svg+xml',
    'ico': 'image/x-icon', 'bmp': 'image/bmp',
    // Documents
    'pdf': 'application/pdf', 'doc': 'application/msword',
    'docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'xls': 'application/vnd.ms-excel',
    'xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    'ppt': 'application/vnd.ms-powerpoint',
    'pptx': 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    // Text
    'txt': 'text/plain', 'md': 'text/markdown', 'json': 'application/json',
    'xml': 'application/xml', 'html': 'text/html', 'css': 'text/css',
    'js': 'text/javascript', 'ts': 'text/typescript',
    // Archives
    'zip': 'application/zip', 'rar': 'application/x-rar-compressed',
    'tar': 'application/x-tar', 'gz': 'application/gzip',
    // Media
    'mp3': 'audio/mpeg', 'wav': 'audio/wav', 'mp4': 'video/mp4',
    'webm': 'video/webm', 'avi': 'video/x-msvideo',
  };
  return mimeTypes[ext] || 'application/octet-stream';
}

async function addFile(filePath: string, category: Category): Promise<VaultFile | null> {
  const spanId = traceStart("file:add", { path: filePath, category });
  if (!session.sessionKey || !session.filesIndex) {
    traceEnd(spanId, { success: false, error: "session locked" });
    return null;
  }

  try {
    // Read the file bytes
    const fileBytes = await readBytes(filePath);

    // Extract filename from path
    const pathParts = filePath.split('/');
    const filename = pathParts[pathParts.length - 1] || 'unknown';

    const id = randomUUID();
    const now = new Date().toISOString();
    const mimeType = getMimeType(filename);

    // Create file entry for index
    const fileEntry: VaultFile = {
      id,
      name: filename,
      size: fileBytes.length,
      mimeType,
      category,
      createdAt: now
    };

    // Add to index
    session.filesIndex.files.unshift(fileEntry);
    await saveFilesIndex();

    // Encrypt and store file content
    const encryptedContent = encryptBytes(fileBytes, session.sessionKey);
    await set(`file:${id}`, encryptedContent);

    traceEnd(spanId, { success: true, fileId: id, size: fileBytes.length });
    return fileEntry;
  } catch (err) {
    traceEnd(spanId, { success: false, error: String(err) });
    error("Failed to add file", { error: String(err) });
    return null;
  }
}

async function getFileContent(id: string): Promise<Uint8Array | null> {
  const spanId = traceStart("file:get", { fileId: id });
  if (!session.sessionKey || !session.filesIndex) {
    traceEnd(spanId, { success: false, error: "session locked" });
    return null;
  }

  const entry = session.filesIndex.files.find(f => f.id === id);
  if (!entry) {
    traceEnd(spanId, { success: false, error: "file not found" });
    return null;
  }

  const encryptedContent = await get<EncryptedData>(`file:${id}`);
  if (!encryptedContent) {
    traceEnd(spanId, { success: false, error: "no encrypted content" });
    return null;
  }

  try {
    const content = decryptBytes(encryptedContent, session.sessionKey);
    traceEnd(spanId, { success: true, size: content.length });
    return content;
  } catch {
    traceEnd(spanId, { success: false, error: "decryption failed" });
    return null;
  }
}

async function deleteFile(id: string): Promise<boolean> {
  const spanId = traceStart("file:delete", { fileId: id });
  if (!session.sessionKey || !session.filesIndex) {
    traceEnd(spanId, { success: false, error: "session locked" });
    return false;
  }

  const entryIndex = session.filesIndex.files.findIndex(f => f.id === id);
  if (entryIndex === -1) {
    traceEnd(spanId, { success: false, error: "file not found" });
    return false;
  }

  // Remove from index
  session.filesIndex.files.splice(entryIndex, 1);
  await saveFilesIndex();

  // Remove encrypted content
  await remove(`file:${id}`);

  traceEnd(spanId, { success: true });
  return true;
}

async function saveFileAs(id: string, savePath: string): Promise<boolean> {
  const content = await getFileContent(id);
  if (!content) return false;

  try {
    await writeBytes(savePath, content);
    return true;
  } catch {
    return false;
  }
}

function getFilteredFiles(category?: string, search?: string): VaultFile[] {
  if (!session.filesIndex) return [];

  let files = session.filesIndex.files;

  // Filter by category
  if (category && category !== "all") {
    files = files.filter(f => f.category === category);
  }

  // Filter by search
  if (search) {
    const searchLower = search.toLowerCase();
    files = files.filter(f => f.name.toLowerCase().includes(searchLower));
  }

  return files;
}

// ============================================================================
// Password Generator
// ============================================================================

function generatePassword(options: {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  numbers: boolean;
  symbols: boolean;
}): string {
  let charset = "";
  if (options.uppercase) charset += "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
  if (options.lowercase) charset += "abcdefghijklmnopqrstuvwxyz";
  if (options.numbers) charset += "0123456789";
  if (options.symbols) charset += "!@#$%^&*()_+-=[]{}|;:,.<>?";

  if (charset.length === 0) {
    charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  }

  const bytes = randomBytes(options.length);
  let password = "";

  for (let i = 0; i < options.length; i++) {
    password += charset[bytes[i] % charset.length];
  }

  return password;
}

// ============================================================================
// Export/Import
// ============================================================================

async function exportVault(): Promise<{ data: string } | null> {
  if (!session.sessionKey || !session.notesIndex) return null;

  const exportData: { notes: Note[]; exportedAt: string } = {
    notes: [],
    exportedAt: new Date().toISOString()
  };

  for (const entry of session.notesIndex.notes) {
    const note = await getNote(entry.id);
    if (note) {
      exportData.notes.push(note);
    }
  }

  // Encrypt the export data with the session key
  const encrypted = encryptData(JSON.stringify(exportData), session.sessionKey);
  return { data: JSON.stringify(encrypted) };
}

async function importVault(encryptedJson: string): Promise<{ success: boolean; count?: number; error?: string }> {
  if (!session.sessionKey) {
    return { success: false, error: "Vault locked" };
  }

  try {
    const encrypted = JSON.parse(encryptedJson) as EncryptedData;
    const decrypted = decryptData(encrypted, session.sessionKey);
    const importData = JSON.parse(decrypted) as { notes: Note[] };

    let count = 0;
    for (const note of importData.notes) {
      await createNote(note.title, note.content, note.category);
      count++;
    }

    return { success: true, count };
  } catch {
    return { success: false, error: "Failed to decrypt import file. Wrong password or corrupted file." };
  }
}

// ============================================================================
// Auto-Lock Timer
// ============================================================================

function startAutoLockTimer(): void {
  setInterval(() => {
    if (!session.unlocked || !mainWindowId) return;

    const idle = Date.now() - session.lastActivity;
    const remaining = LOCK_TIMEOUT_MS - idle;

    if (remaining <= 0) {
      lockVault();
      sendToWindow(mainWindowId, "vault:state", { locked: true, initialized: true });
      notify("Secure Vault", "Vault locked due to inactivity");
    } else if (remaining <= WARNING_BEFORE_MS && remaining > WARNING_BEFORE_MS - 10000) {
      sendToWindow(mainWindowId, "auto-lock:warning", {
        secondsRemaining: Math.ceil(remaining / 1000)
      });
    }
  }, 10000);
}

// ============================================================================
// Main Application
// ============================================================================

infoLog("Starting Secure Vault", { version: "1.0.0" });
// instant("app:startup", { mode: "dev" });  // TODO: ext_trace u128 not supported

// Create main window
const win = await createWindow({
  url: "app://index.html",
  width: 1200,
  height: 800,
  title: "Secure Vault - Locked",
  devtools: true
});

mainWindowId = win.id;
setDefaultWindow(win.id);  // Enable browser console forwarding
// instant("window:created", { windowId: win.id });  // TODO: ext_trace u128 not supported

// Set up application menu
await menu.setAppMenu([
  {
    label: "File",
    submenu: [
      { id: "new-note", label: "New Note", accelerator: "CmdOrCtrl+N" },
      { id: "add-file", label: "Add File...", accelerator: "CmdOrCtrl+O" },
      { label: "-", type: "separator" },
      { id: "export", label: "Export Vault...", accelerator: "CmdOrCtrl+E" },
      { id: "import", label: "Import Vault..." },
      { label: "-", type: "separator" },
      { id: "lock", label: "Lock Vault", accelerator: "CmdOrCtrl+L" }
    ]
  },
  {
    label: "Edit",
    submenu: [
      { id: "cut", label: "Cut", accelerator: "CmdOrCtrl+X" },
      { id: "copy", label: "Copy", accelerator: "CmdOrCtrl+C" },
      { id: "paste", label: "Paste", accelerator: "CmdOrCtrl+V" }
    ]
  },
  {
    label: "Tools",
    submenu: [
      { id: "password-gen", label: "Password Generator", accelerator: "CmdOrCtrl+G" }
    ]
  }
]);

// Set up system tray
await tray.create({
  tooltip: "Secure Vault",
  menu: [
    { id: "tray-lock", label: "Lock Vault" },
    { id: "tray-show", label: "Show Window" },
    { label: "-", type: "separator" },
    { id: "tray-quit", label: "Quit" }
  ]
});

// Start auto-lock timer
startAutoLockTimer();

// Send initial vault state immediately after window creation.
// This now works reliably thanks to the framework-level message queue.
// Messages are queued until the renderer signals it's ready to receive.
const isInit = await isVaultInitialized();
sendToWindow(win.id, "vault:state", {
  unlocked: session.unlocked,
  firstTime: !isInit
});
debug("Sent initial vault:state to renderer", { unlocked: session.unlocked, firstTime: !isInit });

// Handle IPC events from the renderer
for await (const event of windowEvents()) {
  // Reset activity timer on any user action (except ping)
  if (event.channel !== "activity:ping") {
    resetActivityTimer();
  }

  switch (event.channel) {
    // ========================================
    // Vault Operations
    // ========================================

    case "vault:status":
    case "vault:check-setup": {
      const isInit = await isVaultInitialized();
      sendToWindow(win.id, "vault:state", {
        unlocked: session.unlocked,
        firstTime: !isInit
      });
      break;
    }

    case "vault:setup": {
      const { password } = event.payload as { password: string };
      const result = await initializeVault(password);
      // Send unlock-result for both setup and unlock (frontend uses same handler)
      sendToWindow(win.id, "vault:unlock-result", {
        success: result.success,
        error: result.error,
        firstTime: true
      });
      break;
    }

    case "vault:unlock": {
      const { password } = event.payload as { password: string };
      const result = await unlockVault(password);
      sendToWindow(win.id, "vault:unlock-result", {
        success: result.success,
        error: result.error,
        firstTime: false
      });
      break;
    }

    case "vault:lock": {
      lockVault();
      sendToWindow(win.id, "vault:state", { locked: true, initialized: true });
      break;
    }

    // ========================================
    // Notes Operations
    // ========================================

    case "note:list": {
      if (!session.unlocked) {
        sendToWindow(win.id, "note:list-result", { notes: [], error: "Vault locked" });
        break;
      }
      const { category, search } = (event.payload || {}) as { category?: string; search?: string };
      const notes = getFilteredNotes(category, search);
      sendToWindow(win.id, "note:list-result", { notes });
      break;
    }

    case "note:get": {
      if (!session.unlocked) {
        sendToWindow(win.id, "note:get-result", { note: null, error: "Vault locked" });
        break;
      }
      const { id } = event.payload as { id: string };
      const note = await getNote(id);
      sendToWindow(win.id, "note:get-result", { note });
      break;
    }

    case "note:create": {
      if (!session.unlocked) {
        sendToWindow(win.id, "note:saved", { success: false, error: "Vault locked" });
        break;
      }
      // Frontend sends { note: { title, content, category, ... } }
      const { note: noteData } = event.payload as { note: { title: string; content: string; category: Category } };
      const note = await createNote(noteData.title || "Untitled", noteData.content || "", noteData.category || "personal");
      sendToWindow(win.id, "note:saved", { success: !!note, note });
      // Refresh list
      sendToWindow(win.id, "note:list-result", { notes: getFilteredNotes() });
      break;
    }

    case "note:update": {
      if (!session.unlocked) {
        sendToWindow(win.id, "note:saved", { success: false, error: "Vault locked" });
        break;
      }
      // Frontend sends { note: { id, title, content, category, ... } }
      const { note: noteData } = event.payload as { note: Note };
      const success = await updateNote(noteData.id, {
        title: noteData.title,
        content: noteData.content,
        category: noteData.category as Category
      });
      // Return the full updated note
      const updatedNote = success ? await getNote(noteData.id) : null;
      sendToWindow(win.id, "note:saved", { success, note: updatedNote });
      // Refresh list
      sendToWindow(win.id, "note:list-result", { notes: getFilteredNotes() });
      break;
    }

    case "note:delete": {
      if (!session.unlocked) {
        sendToWindow(win.id, "note:deleted", { success: false, error: "Vault locked" });
        break;
      }
      // Frontend sends { noteId: string }
      const { noteId } = event.payload as { noteId: string };
      const success = await deleteNote(noteId);
      sendToWindow(win.id, "note:deleted", { success, noteId });
      // Refresh list
      sendToWindow(win.id, "note:list-result", { notes: getFilteredNotes() });
      break;
    }

    // ========================================
    // File Operations
    // ========================================

    case "file:add": {
      if (!session.unlocked) {
        sendToWindow(win.id, "file:added", { success: false, error: "Vault locked" });
        break;
      }
      // Open file dialog to select file
      const filePaths = await dialog.open({
        title: "Add File to Vault",
        multiple: false
      });

      if (filePaths && filePaths.length > 0) {
        const { category } = (event.payload || {}) as { category?: Category };
        const file = await addFile(filePaths[0], category || "personal");
        sendToWindow(win.id, "file:added", { success: !!file, file });
        // Refresh list
        sendToWindow(win.id, "file:list-result", { files: getFilteredFiles() });
      } else {
        sendToWindow(win.id, "file:added", { success: false, error: "No file selected" });
      }
      break;
    }

    case "file:list": {
      if (!session.unlocked) {
        sendToWindow(win.id, "file:list-result", { files: [], error: "Vault locked" });
        break;
      }
      const { category, search } = (event.payload || {}) as { category?: string; search?: string };
      const files = getFilteredFiles(category, search);
      sendToWindow(win.id, "file:list-result", { files });
      break;
    }

    case "file:get": {
      if (!session.unlocked) {
        sendToWindow(win.id, "file:get-result", { content: null, error: "Vault locked" });
        break;
      }
      const { id } = event.payload as { id: string };
      const content = await getFileContent(id);
      const entry = session.filesIndex?.files.find(f => f.id === id);
      if (content && entry) {
        // Convert to base64 for transfer to frontend
        const base64Content = toBase64(content);
        sendToWindow(win.id, "file:get-result", {
          file: entry,
          content: base64Content,
          mimeType: entry.mimeType
        });
      } else {
        sendToWindow(win.id, "file:get-result", { content: null, error: "File not found" });
      }
      break;
    }

    case "file:save": {
      if (!session.unlocked) {
        sendToWindow(win.id, "file:saved", { success: false, error: "Vault locked" });
        break;
      }
      const { id: fileId } = event.payload as { id: string };
      const entry = session.filesIndex?.files.find(f => f.id === fileId);

      if (!entry) {
        sendToWindow(win.id, "file:saved", { success: false, error: "File not found" });
        break;
      }

      // Open save dialog
      const savePath = await dialog.save({
        title: "Save File As",
        defaultPath: entry.name
      });

      if (savePath) {
        const saved = await saveFileAs(fileId, savePath);
        sendToWindow(win.id, "file:saved", { success: saved });
        if (saved) {
          notify("Secure Vault", `File saved: ${entry.name}`);
        }
      } else {
        sendToWindow(win.id, "file:saved", { success: false, error: "Save cancelled" });
      }
      break;
    }

    case "file:delete": {
      if (!session.unlocked) {
        sendToWindow(win.id, "file:deleted", { success: false, error: "Vault locked" });
        break;
      }
      const { fileId } = event.payload as { fileId: string };
      const deleted = await deleteFile(fileId);
      sendToWindow(win.id, "file:deleted", { success: deleted, fileId });
      // Refresh list
      sendToWindow(win.id, "file:list-result", { files: getFilteredFiles() });
      break;
    }

    // ========================================
    // Password Generator
    // ========================================

    case "password:generate": {
      const options = event.payload as {
        length: number;
        uppercase: boolean;
        lowercase: boolean;
        numbers: boolean;
        symbols: boolean;
      };
      const password = generatePassword(options);
      sendToWindow(win.id, "password:generated", { password });
      break;
    }

    // ========================================
    // Clipboard
    // ========================================

    case "clipboard:copy": {
      const { text } = event.payload as { text: string };
      await clipboard.write(text);
      sendToWindow(win.id, "clipboard:copied", { success: true });
      break;
    }

    // ========================================
    // Export/Import
    // ========================================

    case "export:backup": {
      if (!session.unlocked) {
        sendToWindow(win.id, "export:complete", { success: false, error: "Vault locked" });
        break;
      }
      const exportResult = await exportVault();
      if (exportResult) {
        sendToWindow(win.id, "export:complete", { success: true, data: exportResult.data });
      } else {
        sendToWindow(win.id, "export:complete", { success: false, error: "Export failed" });
      }
      break;
    }

    case "import:backup": {
      const { data } = event.payload as { data: string };
      const importResult = await importVault(data);
      sendToWindow(win.id, "import:complete", importResult);
      if (importResult.success) {
        sendToWindow(win.id, "note:list-result", { notes: getFilteredNotes() });
      }
      break;
    }

    // ========================================
    // Activity Tracking
    // ========================================

    case "activity:ping": {
      // Activity timer already reset above
      break;
    }

    // ========================================
    // Menu Events
    // ========================================

    case "__menu__": {
      const { item_id } = event.payload as { item_id: string };
      switch (item_id) {
        case "new-note":
          sendToWindow(win.id, "menu:new-note", {});
          break;
        case "add-file":
          sendToWindow(win.id, "menu:add-file", {});
          break;
        case "lock":
        case "tray-lock":
          lockVault();
          sendToWindow(win.id, "vault:state", { locked: true, initialized: true });
          break;
        case "export":
          sendToWindow(win.id, "menu:export", {});
          break;
        case "import":
          sendToWindow(win.id, "menu:import", {});
          break;
        case "password-gen":
          sendToWindow(win.id, "menu:password-gen", {});
          break;
        case "tray-show":
          // Window focus would go here
          break;
        case "tray-quit":
          Deno.exit(0);
          break;
      }
      break;
    }

    default:
      warn("Unknown event", { channel: event.channel, payload: event.payload });
  }
}
