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

  export let notes: Note[] = [];
  export let selectedNoteId: string | null = null;
  export let searchQuery = '';

  const dispatch = createEventDispatcher<{
    selectNote: { noteId: string };
    search: { query: string };
  }>();

  const categoryColors: Record<string, string> = {
    personal: 'badge-personal',
    work: 'badge-work',
    financial: 'badge-financial',
    medical: 'badge-medical',
    passwords: 'badge-passwords',
    other: 'badge-other',
  };

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

  function truncateContent(content: string, maxLength = 80): string {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength).trim() + '...';
  }

  function handleSearch(event: Event) {
    const target = event.target as HTMLInputElement;
    dispatch('search', { query: target.value });
  }

  function selectNote(noteId: string) {
    dispatch('selectNote', { noteId });
  }
</script>

<div class="note-list">
  <div class="list-header">
    <div class="search-wrapper">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search notes..."
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
      {notes.length} note{notes.length !== 1 ? 's' : ''}
    </div>
  </div>

  <div class="list-content">
    {#if notes.length === 0}
      <div class="empty-state">
        <div class="empty-state-icon">
          {#if searchQuery}
            🔍
          {:else}
            📝
          {/if}
        </div>
        <p>
          {#if searchQuery}
            No notes found matching "{searchQuery}"
          {:else}
            No notes yet. Create your first note!
          {/if}
        </p>
      </div>
    {:else}
      <ul class="notes">
        {#each notes as note (note.id)}
          <li>
            <button
              class="note-card"
              class:selected={selectedNoteId === note.id}
              on:click={() => selectNote(note.id)}
            >
              <div class="note-header">
                <h4 class="note-title">{note.title || 'Untitled'}</h4>
                <span class="badge {categoryColors[note.category] || 'badge-other'}">
                  {note.category}
                </span>
              </div>
              <p class="note-preview">
                {truncateContent(note.content) || 'No content'}
              </p>
              <div class="note-meta">
                <span class="note-date">{formatDate(note.modifiedAt)}</span>
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .note-list {
    width: 300px;
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

  .notes {
    list-style: none;
  }

  .note-card {
    width: 100%;
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .note-card:hover {
    background-color: var(--bg-hover);
    border-color: var(--border-light);
  }

  .note-card.selected {
    background-color: var(--accent-light);
    border-color: var(--accent-primary);
  }

  .note-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .note-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-preview {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
    margin-bottom: var(--spacing-sm);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .note-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .note-date {
    font-size: 11px;
    color: var(--text-muted);
  }

  .empty-state {
    padding: var(--spacing-xl);
  }
</style>
