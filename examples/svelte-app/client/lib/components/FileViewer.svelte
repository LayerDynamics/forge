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

  export let file: VaultFile | null = null;
  export let content: string | null = null;  // Base64 encoded content
  export let isLoading = false;

  const dispatch = createEventDispatcher<{
    save: { fileId: string };
    delete: { fileId: string };
  }>();

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function isImageFile(mimeType: string): boolean {
    return mimeType.startsWith('image/');
  }

  function isTextFile(mimeType: string): boolean {
    return mimeType.startsWith('text/') ||
           mimeType === 'application/json' ||
           mimeType === 'application/xml';
  }

  function getTextContent(): string {
    if (!content) return '';
    try {
      // Decode base64 to text
      return atob(content);
    } catch {
      return 'Unable to decode file content';
    }
  }

  function getImageSrc(): string {
    if (!content || !file) return '';
    return `data:${file.mimeType};base64,${content}`;
  }

  function handleSave() {
    if (file) {
      dispatch('save', { fileId: file.id });
    }
  }

  function handleDelete() {
    if (file && confirm(`Delete "${file.name}"? This cannot be undone.`)) {
      dispatch('delete', { fileId: file.id });
    }
  }
</script>

<div class="file-viewer">
  {#if !file}
    <div class="no-file">
      <div class="no-file-icon">🔒</div>
      <h3>Select a file to view</h3>
      <p>Choose an encrypted file from the list to preview it securely.</p>
    </div>
  {:else}
    <div class="viewer-header">
      <div class="file-title">
        <h2>{file.name}</h2>
        <span class="file-type">{file.mimeType}</span>
      </div>
      <div class="header-actions">
        <button class="btn btn-primary" on:click={handleSave}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Save to Disk
        </button>
        <button class="btn btn-danger" on:click={handleDelete}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          Delete
        </button>
      </div>
    </div>

    <div class="viewer-content">
      {#if isLoading}
        <div class="loading">
          <div class="spinner"></div>
          <p>Decrypting file...</p>
        </div>
      {:else if content}
        {#if isImageFile(file.mimeType)}
          <div class="image-preview">
            <img src={getImageSrc()} alt={file.name} />
          </div>
        {:else if isTextFile(file.mimeType)}
          <div class="text-preview">
            <pre>{getTextContent()}</pre>
          </div>
        {:else}
          <div class="no-preview">
            <div class="no-preview-icon">📎</div>
            <h3>Preview not available</h3>
            <p>This file type cannot be previewed. Click "Save to Disk" to export it.</p>
          </div>
        {/if}
      {:else}
        <div class="no-preview">
          <div class="no-preview-icon">🔐</div>
          <h3>File encrypted</h3>
          <p>Click the file to load and decrypt its contents.</p>
        </div>
      {/if}
    </div>

    <div class="viewer-footer">
      <div class="file-meta">
        <div class="meta-item">
          <span class="meta-label">Size</span>
          <span class="meta-value">{formatFileSize(file.size)}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Added</span>
          <span class="meta-value">{formatDate(file.createdAt)}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Category</span>
          <span class="meta-value">{file.category}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Encryption</span>
          <span class="meta-value">AES-256-GCM</span>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .file-viewer {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary);
    overflow: hidden;
  }

  .no-file {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--spacing-xl);
  }

  .no-file-icon {
    font-size: 64px;
    margin-bottom: var(--spacing-lg);
  }

  .no-file h3 {
    font-size: 18px;
    color: var(--text-primary);
    margin-bottom: var(--spacing-sm);
  }

  .no-file p {
    color: var(--text-secondary);
    font-size: 14px;
    max-width: 300px;
  }

  .viewer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--border-color);
    background-color: var(--bg-secondary);
  }

  .file-title h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--spacing-xs);
  }

  .file-type {
    font-size: 12px;
    color: var(--text-muted);
  }

  .header-actions {
    display: flex;
    gap: var(--spacing-sm);
  }

  .btn-danger {
    background-color: #dc2626;
    color: white;
    border: none;
  }

  .btn-danger:hover {
    background-color: #b91c1c;
  }

  .viewer-content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
    padding: var(--spacing-lg);
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
  }

  .loading p {
    color: var(--text-secondary);
  }

  .image-preview {
    max-width: 100%;
    max-height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .image-preview img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: var(--radius-md);
    box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  }

  .text-preview {
    width: 100%;
    height: 100%;
    overflow: auto;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
  }

  .text-preview pre {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
  }

  .no-preview {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: var(--spacing-xl);
  }

  .no-preview-icon {
    font-size: 48px;
    margin-bottom: var(--spacing-md);
  }

  .no-preview h3 {
    font-size: 16px;
    color: var(--text-primary);
    margin-bottom: var(--spacing-sm);
  }

  .no-preview p {
    color: var(--text-secondary);
    font-size: 14px;
  }

  .viewer-footer {
    padding: var(--spacing-md) var(--spacing-lg);
    border-top: 1px solid var(--border-color);
    background-color: var(--bg-secondary);
  }

  .file-meta {
    display: flex;
    gap: var(--spacing-xl);
    flex-wrap: wrap;
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .meta-label {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 500;
  }

  .meta-value {
    font-size: 13px;
    color: var(--text-primary);
  }
</style>
