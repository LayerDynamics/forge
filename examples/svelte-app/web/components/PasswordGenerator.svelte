<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';

  const dispatch = createEventDispatcher<{
    copy: { text: string };
  }>();

  // Declare window.host interface
  declare global {
    interface Window {
      runtime: {
        send: (channel: string, data?: unknown) => void;
        on: (channel: string, callback: (data: unknown) => void) => void;
      };
    }
  }

  let password = '';
  let length = 16;
  let includeUppercase = true;
  let includeLowercase = true;
  let includeNumbers = true;
  let includeSymbols = true;
  let isGenerating = false;

  // Password strength calculation
  $: strength = calculateStrength(password);
  $: strengthLabel = getStrengthLabel(strength);
  $: strengthColor = getStrengthColor(strength);

  onMount(() => {
    // Listen for generated password
    window.host.on('password:generated', (data: unknown) => {
      const { password: generatedPassword } = data as { password: string };
      password = generatedPassword;
      isGenerating = false;
    });

    // Generate initial password
    generatePassword();
  });

  function generatePassword() {
    isGenerating = true;
    window.host.send('password:generate', {
      length,
      uppercase: includeUppercase,
      lowercase: includeLowercase,
      numbers: includeNumbers,
      symbols: includeSymbols,
    });
  }

  function copyPassword() {
    if (password) {
      dispatch('copy', { text: password });
    }
  }

  function calculateStrength(pwd: string): number {
    if (!pwd) return 0;

    let score = 0;

    // Length scoring
    if (pwd.length >= 8) score += 1;
    if (pwd.length >= 12) score += 1;
    if (pwd.length >= 16) score += 1;
    if (pwd.length >= 20) score += 1;

    // Character variety scoring
    if (/[a-z]/.test(pwd)) score += 1;
    if (/[A-Z]/.test(pwd)) score += 1;
    if (/[0-9]/.test(pwd)) score += 1;
    if (/[^a-zA-Z0-9]/.test(pwd)) score += 1;

    return Math.min(score, 8);
  }

  function getStrengthLabel(score: number): string {
    if (score <= 2) return 'Weak';
    if (score <= 4) return 'Fair';
    if (score <= 6) return 'Good';
    return 'Strong';
  }

  function getStrengthColor(score: number): string {
    if (score <= 2) return 'var(--error)';
    if (score <= 4) return 'var(--warning)';
    if (score <= 6) return '#22c55e';
    return '#10b981';
  }

  function handleLengthChange(event: Event) {
    const target = event.target as HTMLInputElement;
    length = parseInt(target.value, 10);
  }
</script>

<div class="password-generator">
  <div class="generator-container">
    <div class="generator-header">
      <div class="header-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      </div>
      <h2>Password Generator</h2>
      <p class="header-description">Generate strong, secure passwords</p>
    </div>

    <div class="password-display">
      <div class="password-text" class:generating={isGenerating}>
        {#if isGenerating}
          <span class="generating-text">Generating...</span>
        {:else}
          {password || 'Click generate'}
        {/if}
      </div>
      <div class="password-actions">
        <button
          class="btn btn-ghost btn-icon"
          on:click={copyPassword}
          title="Copy password"
          disabled={!password || isGenerating}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
        </button>
        <button
          class="btn btn-ghost btn-icon"
          on:click={generatePassword}
          title="Generate new password"
          disabled={isGenerating}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:spinning={isGenerating}>
            <polyline points="23 4 23 10 17 10"/>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="strength-meter">
      <div class="strength-bar">
        <div
          class="strength-fill"
          style="width: {(strength / 8) * 100}%; background-color: {strengthColor}"
        ></div>
      </div>
      <span class="strength-label" style="color: {strengthColor}">{strengthLabel}</span>
    </div>

    <div class="generator-options">
      <div class="option-group">
        <label class="option-label" for="length">
          Length: <span class="length-value">{length}</span>
        </label>
        <input
          type="range"
          id="length"
          min="8"
          max="64"
          value={length}
          on:input={handleLengthChange}
          on:change={generatePassword}
          class="length-slider"
        />
        <div class="length-marks">
          <span>8</span>
          <span>24</span>
          <span>40</span>
          <span>64</span>
        </div>
      </div>

      <div class="character-options">
        <label class="checkbox-option">
          <input
            type="checkbox"
            bind:checked={includeUppercase}
            on:change={generatePassword}
          />
          <span class="checkbox-label">
            <span class="option-name">Uppercase</span>
            <span class="option-example">A-Z</span>
          </span>
        </label>

        <label class="checkbox-option">
          <input
            type="checkbox"
            bind:checked={includeLowercase}
            on:change={generatePassword}
          />
          <span class="checkbox-label">
            <span class="option-name">Lowercase</span>
            <span class="option-example">a-z</span>
          </span>
        </label>

        <label class="checkbox-option">
          <input
            type="checkbox"
            bind:checked={includeNumbers}
            on:change={generatePassword}
          />
          <span class="checkbox-label">
            <span class="option-name">Numbers</span>
            <span class="option-example">0-9</span>
          </span>
        </label>

        <label class="checkbox-option">
          <input
            type="checkbox"
            bind:checked={includeSymbols}
            on:change={generatePassword}
          />
          <span class="checkbox-label">
            <span class="option-name">Symbols</span>
            <span class="option-example">!@#$%</span>
          </span>
        </label>
      </div>
    </div>

    <button class="btn btn-primary btn-lg w-full" on:click={generatePassword} disabled={isGenerating}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="23 4 23 10 17 10"/>
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
      </svg>
      Generate Password
    </button>

    <div class="generator-tips">
      <h4>Tips for secure passwords:</h4>
      <ul>
        <li>Use at least 16 characters</li>
        <li>Include a mix of character types</li>
        <li>Never reuse passwords across sites</li>
        <li>Store passwords in a secure vault</li>
      </ul>
    </div>
  </div>
</div>

<style>
  .password-generator {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    overflow-y: auto;
  }

  .generator-container {
    width: 100%;
    max-width: 500px;
  }

  .generator-header {
    text-align: center;
    margin-bottom: var(--spacing-xl);
  }

  .header-icon {
    width: 64px;
    height: 64px;
    margin: 0 auto var(--spacing-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--accent-primary) 0%, #9333ea 100%);
    border-radius: var(--radius-xl);
    color: white;
  }

  .header-description {
    color: var(--text-secondary);
    margin-top: var(--spacing-sm);
  }

  .password-display {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    margin-bottom: var(--spacing-md);
  }

  .password-text {
    flex: 1;
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
    font-size: 18px;
    word-break: break-all;
    color: var(--text-primary);
  }

  .password-text.generating {
    color: var(--text-muted);
  }

  .generating-text {
    animation: pulse 1s infinite;
  }

  .password-actions {
    display: flex;
    gap: var(--spacing-xs);
    flex-shrink: 0;
  }

  .spinning {
    animation: spin 0.5s linear infinite;
  }

  .strength-meter {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-xl);
  }

  .strength-bar {
    flex: 1;
    height: 6px;
    background-color: var(--bg-tertiary);
    border-radius: 3px;
    overflow: hidden;
  }

  .strength-fill {
    height: 100%;
    border-radius: 3px;
    transition: all var(--transition-normal);
  }

  .strength-label {
    font-size: 12px;
    font-weight: 500;
    min-width: 50px;
    text-align: right;
  }

  .generator-options {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
  }

  .option-group {
    margin-bottom: var(--spacing-lg);
  }

  .option-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: var(--spacing-sm);
  }

  .length-value {
    color: var(--accent-primary);
    font-weight: 600;
  }

  .length-slider {
    width: 100%;
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
    cursor: pointer;
  }

  .length-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    background: var(--accent-primary);
    border-radius: 50%;
    cursor: pointer;
    transition: transform var(--transition-fast);
  }

  .length-slider::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }

  .length-marks {
    display: flex;
    justify-content: space-between;
    margin-top: var(--spacing-xs);
    font-size: 11px;
    color: var(--text-muted);
  }

  .character-options {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-sm);
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
    background-color: var(--bg-tertiary);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }

  .checkbox-option:hover {
    background-color: var(--bg-hover);
  }

  .checkbox-option input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent-primary);
    cursor: pointer;
  }

  .checkbox-label {
    display: flex;
    flex-direction: column;
  }

  .option-name {
    font-size: 13px;
    font-weight: 500;
  }

  .option-example {
    font-size: 11px;
    color: var(--text-muted);
    font-family: monospace;
  }

  .generator-tips {
    margin-top: var(--spacing-xl);
    padding: var(--spacing-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
  }

  .generator-tips h4 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-sm);
  }

  .generator-tips ul {
    list-style: none;
    font-size: 12px;
    color: var(--text-muted);
  }

  .generator-tips li {
    padding: var(--spacing-xs) 0;
    padding-left: var(--spacing-md);
    position: relative;
  }

  .generator-tips li::before {
    content: '•';
    position: absolute;
    left: 0;
    color: var(--accent-primary);
  }
</style>
