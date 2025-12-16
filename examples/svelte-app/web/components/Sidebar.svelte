<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  type Category = 'all' | 'personal' | 'work' | 'financial' | 'medical' | 'passwords' | 'other';
  type View = 'notes' | 'generator';

  export let selectedCategory: Category = 'all';
  export let currentView: View = 'notes';

  const dispatch = createEventDispatcher<{
    categoryChange: { category: Category };
    viewChange: { view: View };
    newNote: void;
    lock: void;
    export: void;
    import: void;
  }>();

  const categories: { id: Category; label: string; icon: string }[] = [
    { id: 'all', label: 'All Notes', icon: '📋' },
    { id: 'personal', label: 'Personal', icon: '👤' },
    { id: 'work', label: 'Work', icon: '💼' },
    { id: 'financial', label: 'Financial', icon: '💰' },
    { id: 'medical', label: 'Medical', icon: '🏥' },
    { id: 'passwords', label: 'Passwords', icon: '🔑' },
    { id: 'other', label: 'Other', icon: '📁' },
  ];

  function selectCategory(category: Category) {
    dispatch('categoryChange', { category });
    dispatch('viewChange', { view: 'notes' });
  }

  function selectView(view: View) {
    dispatch('viewChange', { view });
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="logo">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
      </svg>
      <span>Secure Vault</span>
    </div>
  </div>

  <div class="sidebar-content">
    <button class="btn btn-primary w-full" on:click={() => dispatch('newNote')}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New Note
    </button>

    <nav class="nav-section">
      <h3 class="nav-title">Categories</h3>
      <ul class="nav-list">
        {#each categories as category}
          <li>
            <button
              class="nav-item"
              class:active={selectedCategory === category.id && currentView === 'notes'}
              on:click={() => selectCategory(category.id)}
            >
              <span class="nav-icon">{category.icon}</span>
              <span class="nav-label">{category.label}</span>
            </button>
          </li>
        {/each}
      </ul>
    </nav>

    <nav class="nav-section">
      <h3 class="nav-title">Tools</h3>
      <ul class="nav-list">
        <li>
          <button
            class="nav-item"
            class:active={currentView === 'generator'}
            on:click={() => selectView('generator')}
          >
            <span class="nav-icon">🎲</span>
            <span class="nav-label">Password Generator</span>
          </button>
        </li>
      </ul>
    </nav>
  </div>

  <div class="sidebar-footer">
    <div class="footer-actions">
      <button class="btn btn-ghost btn-sm" on:click={() => dispatch('export')} title="Export Backup">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17 8 12 3 7 8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        Export
      </button>
      <button class="btn btn-ghost btn-sm" on:click={() => dispatch('import')} title="Import Backup">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        Import
      </button>
    </div>

    <button class="btn btn-secondary w-full" on:click={() => dispatch('lock')}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
      </svg>
      Lock Vault
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 260px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .sidebar-header {
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border-color);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-weight: 600;
    font-size: 16px;
    color: var(--accent-primary);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-md);
  }

  .nav-section {
    margin-top: var(--spacing-lg);
  }

  .nav-section:first-of-type {
    margin-top: var(--spacing-md);
  }

  .nav-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: var(--spacing-sm);
    padding: 0 var(--spacing-sm);
  }

  .nav-list {
    list-style: none;
  }

  .nav-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: 14px;
    color: var(--text-secondary);
    background: none;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
  }

  .nav-item:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background-color: var(--accent-light);
    color: var(--accent-primary);
  }

  .nav-icon {
    width: 20px;
    text-align: center;
  }

  .nav-label {
    flex: 1;
  }

  .sidebar-footer {
    padding: var(--spacing-md);
    border-top: 1px solid var(--border-color);
  }

  .footer-actions {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .footer-actions .btn {
    flex: 1;
  }
</style>
