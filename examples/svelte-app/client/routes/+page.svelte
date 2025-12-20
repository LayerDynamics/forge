<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import LockScreen from '$lib/components/LockScreen.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import NoteList from '$lib/components/NoteList.svelte';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import PasswordGenerator from '$lib/components/PasswordGenerator.svelte';
  import FileList from '$lib/components/FileList.svelte';
  import FileViewer from '$lib/components/FileViewer.svelte';

  // Types
  interface Note {
    id: string;
    title: string;
    content: string;
    category: string;
    createdAt: string;
    modifiedAt: string;
  }

  interface VaultFile {
    id: string;
    name: string;
    size: number;
    mimeType: string;
    category: string;
    createdAt: string;
  }

  type Category = 'all' | 'personal' | 'work' | 'financial' | 'medical' | 'passwords' | 'other';
  type View = 'notes' | 'files' | 'generator';

  // Declare window.host interface
  declare global {
    interface Window {
      runtime: {
        send: (channel: string, data?: unknown) => void;
        on: (channel: string, callback: (data: unknown) => void) => void;
      };
    }
  }

  // State
  let isUnlocked = false;
  let isFirstTime = true;
  let isLoading = false;  // No loading spinner on startup - framework queues messages until ready
  let error = '';
  let toast: { message: string; type: 'success' | 'error' | 'warning' } | null = null;

  // Vault state - Notes
  let notes: Note[] = [];
  let selectedNoteId: string | null = null;
  let selectedNoteContent: Note | null = null;  // Full note with content

  // Vault state - Files
  let files: VaultFile[] = [];
  let selectedFileId: string | null = null;
  let selectedFileContent: string | null = null;  // Base64 encoded content
  let isFileLoading = false;

  // Shared state
  let selectedCategory: Category = 'all';
  let searchQuery = '';
  let fileSearchQuery = '';
  let currentView: View = 'notes';

  // Activity tracking for auto-lock
  let activityInterval: number;

  // Computed - Notes
  $: selectedNote = selectedNoteContent && selectedNoteContent.id === selectedNoteId
    ? selectedNoteContent
    : notes.find(n => n.id === selectedNoteId) || null;
  $: filteredNotes = notes.filter(note => {
    const matchesCategory = selectedCategory === 'all' || note.category === selectedCategory;
    const matchesSearch = !searchQuery ||
      note.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      note.content.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  // Computed - Files
  $: selectedFile = files.find(f => f.id === selectedFileId) || null;
  $: filteredFiles = files.filter(file => {
    const matchesSearch = !fileSearchQuery ||
      file.name.toLowerCase().includes(fileSearchQuery.toLowerCase());
    return matchesSearch;
  });

  onMount(() => {
    setupEventListeners();
    // No need to call checkVaultStatus() - backend sends vault:state immediately
    // and framework queues it until renderer is ready
    startActivityTracking();
  });

  onDestroy(() => {
    if (activityInterval) {
      clearInterval(activityInterval);
    }
  });

  function setupEventListeners() {
    // Vault state events - received immediately thanks to framework message queue
    window.host.on('vault:state', (data: unknown) => {
      const { unlocked, firstTime } = data as { unlocked: boolean; firstTime: boolean };
      isUnlocked = unlocked;
      isFirstTime = firstTime;
      if (unlocked) {
        loadNotes();
        loadFiles();
      }
    });

    window.host.on('vault:unlock-result', (data: unknown) => {
      const { success, error: err, firstTime } = data as { success: boolean; error?: string; firstTime?: boolean };
      isLoading = false;
      if (success) {
        isUnlocked = true;
        isFirstTime = firstTime ?? false;
        loadNotes();
        loadFiles();
        showToast('Vault unlocked', 'success');
      } else {
        error = err || 'Failed to unlock vault';
        showToast(error, 'error');
      }
    });

    // Note events
    window.host.on('note:list-result', (data: unknown) => {
      const { notes: loadedNotes } = data as { notes: Note[] };
      notes = loadedNotes || [];
    });

    window.host.on('note:saved', (data: unknown) => {
      const { success, note } = data as { success: boolean; note?: Note };
      if (success && note) {
        const index = notes.findIndex(n => n.id === note.id);
        if (index >= 0) {
          notes[index] = note;
          notes = [...notes];
        } else {
          notes = [...notes, note];
        }
        showToast('Note saved', 'success');
      }
    });

    window.host.on('note:deleted', (data: unknown) => {
      const { success, noteId } = data as { success: boolean; noteId: string };
      if (success) {
        notes = notes.filter(n => n.id !== noteId);
        if (selectedNoteId === noteId) {
          selectedNoteId = null;
          selectedNoteContent = null;
        }
        showToast('Note deleted', 'success');
      }
    });

    // Handle full note content response
    window.host.on('note:get-result', (data: unknown) => {
      const { note } = data as { note: Note | null };
      if (note) {
        selectedNoteContent = note;
      }
    });

    // Auto-lock warning
    window.host.on('auto-lock:warning', () => {
      showToast('Vault will lock in 30 seconds due to inactivity', 'warning');
    });

    // Clipboard events
    window.host.on('clipboard:copied', (data: unknown) => {
      const { success } = data as { success: boolean };
      if (success) {
        showToast('Copied to clipboard', 'success');
      }
    });

    // Export/Import events
    window.host.on('export:result', (data: unknown) => {
      const { success, error: err } = data as { success: boolean; error?: string };
      if (success) {
        showToast('Backup exported successfully', 'success');
      } else {
        showToast(err || 'Export failed', 'error');
      }
    });

    window.host.on('import:result', (data: unknown) => {
      const { success, error: err, count } = data as { success: boolean; error?: string; count?: number };
      if (success) {
        loadNotes();
        showToast(`Imported ${count} notes successfully`, 'success');
      } else {
        showToast(err || 'Import failed', 'error');
      }
    });

    // File events
    window.host.on('file:list-result', (data: unknown) => {
      const { files: loadedFiles } = data as { files: VaultFile[] };
      files = loadedFiles || [];
    });

    window.host.on('file:added', (data: unknown) => {
      const { success, file } = data as { success: boolean; file?: VaultFile };
      if (success && file) {
        files = [file, ...files];
        showToast(`Added "${file.name}" to vault`, 'success');
      } else {
        showToast('Failed to add file', 'error');
      }
    });

    window.host.on('file:get-result', (data: unknown) => {
      const { content, file } = data as { content: string | null; file?: VaultFile };
      isFileLoading = false;
      if (content) {
        selectedFileContent = content;
      }
    });

    window.host.on('file:saved', (data: unknown) => {
      const { success } = data as { success: boolean };
      if (success) {
        showToast('File saved to disk', 'success');
      }
    });

    window.host.on('file:deleted', (data: unknown) => {
      const { success, fileId } = data as { success: boolean; fileId: string };
      if (success) {
        files = files.filter(f => f.id !== fileId);
        if (selectedFileId === fileId) {
          selectedFileId = null;
          selectedFileContent = null;
        }
        showToast('File deleted', 'success');
      }
    });

    // Menu events
    window.host.on('menu:add-file', () => {
      handleAddFile();
    });
  }

  function loadNotes() {
    window.host.send('note:list', { category: selectedCategory === 'all' ? undefined : selectedCategory });
  }

  function loadFiles() {
    window.host.send('file:list');
  }

  function startActivityTracking() {
    // Send activity ping every 30 seconds
    activityInterval = setInterval(() => {
      if (isUnlocked) {
        window.host.send('activity:ping');
      }
    }, 30000) as unknown as number;

    // Track user activity
    const trackActivity = () => {
      if (isUnlocked) {
        window.host.send('activity:ping');
      }
    };

    document.addEventListener('mousemove', trackActivity);
    document.addEventListener('keydown', trackActivity);
    document.addEventListener('click', trackActivity);
  }

  function handleUnlock(event: CustomEvent<{ password: string }>) {
    isLoading = true;
    error = '';
    window.host.send('vault:unlock', { password: event.detail.password });
  }

  function handleSetup(event: CustomEvent<{ password: string }>) {
    isLoading = true;
    error = '';
    window.host.send('vault:setup', { password: event.detail.password });
  }

  function handleLock() {
    window.host.send('vault:lock');
    isUnlocked = false;
    notes = [];
    selectedNoteId = null;
    selectedNoteContent = null;
    files = [];
    selectedFileId = null;
    selectedFileContent = null;
    showToast('Vault locked', 'success');
  }

  function handleSelectNote(event: CustomEvent<{ noteId: string }>) {
    selectedNoteId = event.detail.noteId;
    selectedNoteContent = null;  // Clear previous content
    currentView = 'notes';
    // Fetch full note content
    window.host.send('note:get', { id: event.detail.noteId });
  }

  function handleNewNote() {
    const newNote: Note = {
      id: '',
      title: 'Untitled Note',
      content: '',
      category: selectedCategory === 'all' ? 'personal' : selectedCategory,
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString()
    };
    window.host.send('note:create', { note: newNote });
  }

  function handleSaveNote(event: CustomEvent<{ note: Note }>) {
    window.host.send('note:update', { note: event.detail.note });
  }

  function handleDeleteNote(event: CustomEvent<{ noteId: string }>) {
    window.host.send('note:delete', { noteId: event.detail.noteId });
  }

  function handleCopyToClipboard(event: CustomEvent<{ text: string }>) {
    window.host.send('clipboard:copy', { text: event.detail.text });
  }

  function handleExport() {
    window.host.send('export:backup');
  }

  function handleImport() {
    window.host.send('import:backup');
  }

  // File handlers
  function handleAddFile() {
    window.host.send('file:add', { category: 'personal' });
  }

  function handleSelectFile(event: CustomEvent<{ fileId: string }>) {
    selectedFileId = event.detail.fileId;
    selectedFileContent = null;
    isFileLoading = true;
    currentView = 'files';
    window.host.send('file:get', { id: event.detail.fileId });
  }

  function handleSaveFile(event: CustomEvent<{ fileId: string }>) {
    window.host.send('file:save', { id: event.detail.fileId });
  }

  function handleDeleteFile(event: CustomEvent<{ fileId: string }>) {
    window.host.send('file:delete', { fileId: event.detail.fileId });
  }

  function handleFileSearch(event: CustomEvent<{ query: string }>) {
    fileSearchQuery = event.detail.query;
  }

  function handleCategoryChange(event: CustomEvent<{ category: Category }>) {
    selectedCategory = event.detail.category;
    loadNotes();
  }

  function handleViewChange(event: CustomEvent<{ view: View }>) {
    currentView = event.detail.view;
    if (currentView === 'generator') {
      selectedNoteId = null;
    }
  }

  function handleSearch(event: CustomEvent<{ query: string }>) {
    searchQuery = event.detail.query;
  }

  function showToast(message: string, type: 'success' | 'error' | 'warning') {
    toast = { message, type };
    setTimeout(() => {
      toast = null;
    }, 3000);
  }
</script>

<div class="app">
  {#if !isUnlocked}
    <LockScreen
      {isFirstTime}
      {error}
      on:unlock={handleUnlock}
      on:setup={handleSetup}
    />
  {:else}
    <div class="vault-layout">
      <Sidebar
        {selectedCategory}
        {currentView}
        on:categoryChange={handleCategoryChange}
        on:viewChange={handleViewChange}
        on:newNote={handleNewNote}
        on:addFile={handleAddFile}
        on:lock={handleLock}
        on:export={handleExport}
        on:import={handleImport}
      />

      <main class="main-content">
        {#if currentView === 'notes'}
          <div class="notes-panel">
            <NoteList
              notes={filteredNotes}
              {selectedNoteId}
              {searchQuery}
              on:selectNote={handleSelectNote}
              on:search={handleSearch}
            />

            <NoteEditor
              note={selectedNote}
              on:save={handleSaveNote}
              on:delete={handleDeleteNote}
              on:copy={handleCopyToClipboard}
            />
          </div>
        {:else if currentView === 'files'}
          <div class="files-panel">
            <FileList
              files={filteredFiles}
              {selectedFileId}
              searchQuery={fileSearchQuery}
              on:selectFile={handleSelectFile}
              on:search={handleFileSearch}
              on:saveFile={handleSaveFile}
              on:deleteFile={handleDeleteFile}
            />

            <FileViewer
              file={selectedFile}
              content={selectedFileContent}
              isLoading={isFileLoading}
              on:save={handleSaveFile}
              on:delete={handleDeleteFile}
            />
          </div>
        {:else if currentView === 'generator'}
          <PasswordGenerator on:copy={handleCopyToClipboard} />
        {/if}
      </main>
    </div>
  {/if}

  {#if toast}
    <div class="toast toast-{toast.type}">
      {toast.message}
    </div>
  {/if}
</div>

<style>
  .app {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary);
    color: var(--text-primary);
  }

  .vault-layout {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .notes-panel {
    display: flex;
    width: 100%;
    height: 100%;
  }

  .files-panel {
    display: flex;
    width: 100%;
    height: 100%;
  }
</style>
