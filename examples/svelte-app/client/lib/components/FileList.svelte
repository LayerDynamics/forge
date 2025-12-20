<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  interface VaultFile {
    id: string;
    name: string;
    size: number;
    mimeType: string;
    category: string;
    createdAt: string;
  }

  export let files: VaultFile[] = [];
  export let selectedFileId: string | null = null;
  export let searchQuery = '';

  const dispatch = createEventDispatcher<{
    selectFile: { fileId: string };
    search: { query: string };
    deleteFile: { fileId: string };
    saveFile: { fileId: string };
  }>();

  const categoryColors: Record<string, string> = {
    personal: 'badge-personal',
    work: 'badge-work',
    financial: 'badge-financial',
    medical: 'badge-medical',
    passwords: 'badge-passwords',
    other: 'badge-other',
  };

  const fileIcons: Record<string, string> = {
    'image': '🖼️',
    'video': '🎬',
    'audio': '🎵',
    'text': '📄',
    'application/pdf': '📕',
    'application/zip': '📦',
    'application/x-rar': '📦',
    'application/json': '📋',
    'default': '📎'
  };

  function getFileIcon(mimeType: string): string {
    if (mimeType.startsWith('image/')) return fileIcons['image'];
    if (mimeType.startsWith('video/')) return fileIcons['video'];
    if (mimeType.startsWith('audio/')) return fileIcons['audio'];
    if (mimeType.startsWith('text/')) return fileIcons['text'];
    return fileIcons[mimeType] || fileIcons['default'];
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) {
      return 'Today';
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return `${days} days ago`;
    } else {
      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
  }

  function handleSearch(event: Event) {
    const target = event.target as HTMLInputElement;
    dispatch('search', { query: target.value });
  }

  function selectFile(fileId: string) {
    dispatch('selectFile', { fileId });
  }

  function handleSave(event: Event, fileId: string) {
    event.stopPropagation();
    dispatch('saveFile', { fileId });
  }

  function handleDelete(event: Event, fileId: string) {
    event.stopPropagation();
    dispatch('deleteFile', { fileId });
  }
</script>

<div class="file-list">
  <div class="list-header">
    <div class="search-wrapper">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search files..."
        value={searchQuery}
        on:input={handleSearch}
      />
      {#if searchQuery}
        <button class="search-clear" on:click={() => dispatch('search', { query: '' })}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    </div>
    <div class="list-count">
      {files.length} file{files.length !== 1 ? 's' : ''}
    </div>
  </div>

  <div class="list-content">
    {#if files.length === 0}
      <div class="empty-state">
        <div class="empty-state-icon">
          {#if searchQuery}
            🔍
          {:else}
            📁
          {/if}
        </div>
        <p>
          {#if searchQuery}
            No files found matching "{searchQuery}"
          {:else}
            No encrypted files yet. Add your first file!
          {/if}
        </p>
      </div>
    {:else}
      <ul class="files">
        {#each files as file (file.id)}
          <li>
            <button
              class="file-card"
              class:selected={selectedFileId === file.id}
              on:click={() => selectFile(file.id)}
            >
              <div class="file-icon">
                {getFileIcon(file.mimeType)}
              </div>
              <div class="file-info">
                <div class="file-header">
                  <h4 class="file-name" title={file.name}>{file.name}</h4>
                </div>
                <div class="file-meta">
                  <span class="file-size">{formatFileSize(file.size)}</span>
                  <span class="file-date">{formatDate(file.createdAt)}</span>
                </div>
                <span class="badge {categoryColors[file.category] || 'badge-other'}">
                  {file.category}
                </span>
              </div>
              <div class="file-actions">
                <button
                  class="action-btn"
                  title="Save to disk"
                  on:click={(e) => handleSave(e, file.id)}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="7 10 12 15 17 10"/>
                    <line x1="12" y1="15" x2="12" y2="3"/>
                  </svg>
                </button>
                <button
                  class="action-btn action-btn-danger"
                  title="Delete file"
                  on:click={(e) => handleDelete(e, file.id)}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                  </svg>
                </button>
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .file-list {
    width: 350px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary);
    border-right: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .list-header {
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border-color);
  }

  .search-wrapper {
    position: relative;
    margin-bottom: var(--spacing-sm);
  }

  .search-icon {
    position: absolute;
    left: var(--spacing-md);
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    padding-left: 40px;
    padding-right: 36px;
    font-size: 14px;
    color: var(--text-primary);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    outline: none;
    transition: all var(--transition-fast);
  }

  .search-input:focus {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 3px var(--accent-light);
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-clear {
    position: absolute;
    right: var(--spacing-sm);
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: var(--spacing-xs);
    border-radius: var(--radius-sm);
  }

  .search-clear:hover {
    color: var(--text-primary);
    background-color: var(--bg-hover);
  }

  .list-count {
    font-size: 12px;
    color: var(--text-muted);
  }

  .list-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-sm);
  }

  .files {
    list-style: none;
  }

  .file-card {
    width: 100%;
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .file-card:hover {
    background-color: var(--bg-hover);
    border-color: var(--border-light);
  }

  .file-card.selected {
    background-color: var(--accent-light);
    border-color: var(--accent-primary);
  }

  .file-icon {
    font-size: 28px;
    flex-shrink: 0;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-header {
    margin-bottom: var(--spacing-xs);
  }

  .file-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    display: flex;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-xs);
  }

  .file-size,
  .file-date {
    font-size: 11px;
    color: var(--text-muted);
  }

  .file-actions {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .file-card:hover .file-actions {
    opacity: 1;
  }

  .action-btn {
    padding: var(--spacing-xs);
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .action-btn:hover {
    background: var(--accent-light);
    color: var(--accent-primary);
    border-color: var(--accent-primary);
  }

  .action-btn-danger:hover {
    background: #fef2f2;
    color: #dc2626;
    border-color: #dc2626;
  }

  .empty-state {
    padding: var(--spacing-xl);
    text-align: center;
  }

  .empty-state-icon {
    font-size: 48px;
    margin-bottom: var(--spacing-md);
  }

  .empty-state p {
    color: var(--text-secondary);
    font-size: 14px;
  }
</style>
