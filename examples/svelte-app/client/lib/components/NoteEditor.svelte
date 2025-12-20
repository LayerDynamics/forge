<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  interface Note {
    id: string;
    title: string;
    content: string;
    category: string;
    createdAt: string;
    modifiedAt: string;
  }

  export let note: Note | null = null;

  const dispatch = createEventDispatcher<{
    save: { note: Note };
    delete: { noteId: string };
    copy: { text: string };
  }>();

  const categories = [
    { id: 'personal', label: 'Personal' },
    { id: 'work', label: 'Work' },
    { id: 'financial', label: 'Financial' },
    { id: 'medical', label: 'Medical' },
    { id: 'passwords', label: 'Passwords' },
    { id: 'other', label: 'Other' },
  ];

  let title = '';
  let content = '';
  let category = 'personal';
  let hasChanges = false;
  let showDeleteConfirm = false;

  // Update local state when note changes
  $: if (note) {
    title = note.title;
    content = note.content;
    category = note.category;
    hasChanges = false;
    showDeleteConfirm = false;
  }

  function handleTitleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    title = target.value;
    hasChanges = true;
  }

  function handleContentChange(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    content = target.value;
    hasChanges = true;
  }

  function handleCategoryChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    category = target.value;
    hasChanges = true;
  }

  function saveNote() {
    if (!note) return;

    const updatedNote: Note = {
      ...note,
      title,
      content,
      category,
      modifiedAt: new Date().toISOString(),
    };

    dispatch('save', { note: updatedNote });
    hasChanges = false;
  }

  function deleteNote() {
    if (!note) return;
    dispatch('delete', { noteId: note.id });
    showDeleteConfirm = false;
  }

  function copyContent() {
    if (content) {
      dispatch('copy', { text: content });
    }
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  }
</script>

<div class="note-editor">
  {#if note}
    <div class="editor-header">
      <div class="editor-title-row">
        <input
          type="text"
          class="title-input"
          value={title}
          on:input={handleTitleChange}
          placeholder="Note title..."
        />
        <div class="editor-actions">
          <button
            class="btn btn-ghost btn-icon"
            on:click={copyContent}
            title="Copy content"
            disabled={!content}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
          </button>
          <button
            class="btn btn-ghost btn-icon"
            on:click={() => showDeleteConfirm = true}
            title="Delete note"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
          </button>
        </div>
      </div>

      <div class="editor-meta">
        <select class="category-select" value={category} on:change={handleCategoryChange}>
          {#each categories as cat}
            <option value={cat.id}>{cat.label}</option>
          {/each}
        </select>
        <span class="meta-date">Modified {formatDate(note.modifiedAt)}</span>
      </div>
    </div>

    <div class="editor-content">
      <textarea
        class="content-textarea"
        value={content}
        on:input={handleContentChange}
        placeholder="Start writing your note..."
      ></textarea>
    </div>

    <div class="editor-footer">
      <div class="footer-info">
        {#if hasChanges}
          <span class="unsaved-indicator">Unsaved changes</span>
        {:else}
          <span class="saved-indicator">All changes saved</span>
        {/if}
      </div>
      <button
        class="btn btn-primary"
        on:click={saveNote}
        disabled={!hasChanges}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
          <polyline points="17 21 17 13 7 13 7 21"/>
          <polyline points="7 3 7 8 15 8"/>
        </svg>
        Save Note
      </button>
    </div>

    {#if showDeleteConfirm}
      <div class="delete-confirm-overlay">
        <div class="delete-confirm-dialog">
          <h3>Delete Note</h3>
          <p>Are you sure you want to delete "{title || 'Untitled'}"? This action cannot be undone.</p>
          <div class="dialog-actions">
            <button class="btn btn-secondary" on:click={() => showDeleteConfirm = false}>
              Cancel
            </button>
            <button class="btn btn-danger" on:click={deleteNote}>
              Delete
            </button>
          </div>
        </div>
      </div>
    {/if}
  {:else}
    <div class="empty-editor">
      <div class="empty-state-icon">📝</div>
      <h3>Select a note</h3>
      <p>Choose a note from the list to view and edit its contents, or create a new note.</p>
    </div>
  {/if}
</div>

<style>
  .note-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary);
    overflow: hidden;
    position: relative;
  }

  .editor-header {
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--border-color);
  }

  .editor-title-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .title-input {
    flex: 1;
    font-size: 24px;
    font-weight: 600;
    color: var(--text-primary);
    background: none;
    border: none;
    outline: none;
    padding: 0;
  }

  .title-input::placeholder {
    color: var(--text-muted);
  }

  .editor-actions {
    display: flex;
    gap: var(--spacing-xs);
  }

  .editor-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .category-select {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: 12px;
    color: var(--text-primary);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    outline: none;
    cursor: pointer;
  }

  .meta-date {
    font-size: 12px;
    color: var(--text-muted);
  }

  .editor-content {
    flex: 1;
    padding: var(--spacing-lg);
    overflow-y: auto;
  }

  .content-textarea {
    width: 100%;
    height: 100%;
    min-height: 300px;
    font-size: 15px;
    line-height: 1.7;
    color: var(--text-primary);
    background: none;
    border: none;
    outline: none;
    resize: none;
    font-family: inherit;
  }

  .content-textarea::placeholder {
    color: var(--text-muted);
  }

  .editor-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    border-top: 1px solid var(--border-color);
    background-color: var(--bg-secondary);
  }

  .footer-info {
    font-size: 12px;
  }

  .unsaved-indicator {
    color: var(--warning);
  }

  .saved-indicator {
    color: var(--text-muted);
  }

  .empty-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    text-align: center;
    color: var(--text-muted);
  }

  .empty-editor h3 {
    margin-top: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
    color: var(--text-secondary);
  }

  .empty-editor p {
    max-width: 300px;
  }

  .delete-confirm-overlay {
    position: absolute;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn var(--transition-fast);
  }

  .delete-confirm-dialog {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    max-width: 400px;
    width: 90%;
    animation: slideUp var(--transition-normal);
  }

  .delete-confirm-dialog h3 {
    margin-bottom: var(--spacing-sm);
    color: var(--error);
  }

  .delete-confirm-dialog p {
    color: var(--text-secondary);
    margin-bottom: var(--spacing-lg);
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
  }
</style>
