<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let isFirstTime = true;
  export let error = '';

  const dispatch = createEventDispatcher<{
    unlock: { password: string };
    setup: { password: string };
  }>();

  let password = '';
  let confirmPassword = '';
  let showPassword = false;
  let validationError = '';

  function handleSubmit() {
    validationError = '';

    if (!password) {
      validationError = 'Password is required';
      return;
    }

    if (isFirstTime) {
      if (password.length < 8) {
        validationError = 'Password must be at least 8 characters';
        return;
      }
      if (password !== confirmPassword) {
        validationError = 'Passwords do not match';
        return;
      }
      dispatch('setup', { password });
    } else {
      dispatch('unlock', { password });
    }
  }

  function toggleShowPassword() {
    showPassword = !showPassword;
  }
</script>

<div class="lock-screen">
  <div class="lock-container animate-fade-in">
    <div class="lock-icon">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
      </svg>
    </div>

    <h1 class="lock-title">Secure Vault</h1>
    <p class="lock-subtitle">
      {#if isFirstTime}
        Create a master password to protect your vault
      {:else}
        Enter your master password to unlock
      {/if}
    </p>

    <form class="lock-form" on:submit|preventDefault={handleSubmit}>
      <div class="form-group">
        <label class="label" for="password">Master Password</label>
        <div class="password-input-wrapper">
          {#if showPassword}
            <input
              id="password"
              type="text"
              class="input input-lg"
              bind:value={password}
              placeholder="Enter your password"
              autocomplete={isFirstTime ? 'new-password' : 'current-password'}
            />
          {:else}
            <input
              id="password"
              type="password"
              class="input input-lg"
              bind:value={password}
              placeholder="Enter your password"
              autocomplete={isFirstTime ? 'new-password' : 'current-password'}
            />
          {/if}
          <button
            type="button"
            class="toggle-password"
            on:click={toggleShowPassword}
            aria-label={showPassword ? 'Hide password' : 'Show password'}
          >
            {#if showPassword}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </svg>
            {:else}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            {/if}
          </button>
        </div>
      </div>

      {#if isFirstTime}
        <div class="form-group">
          <label class="label" for="confirm-password">Confirm Password</label>
          {#if showPassword}
            <input
              id="confirm-password"
              type="text"
              class="input input-lg"
              bind:value={confirmPassword}
              placeholder="Confirm your password"
              autocomplete="new-password"
            />
          {:else}
            <input
              id="confirm-password"
              type="password"
              class="input input-lg"
              bind:value={confirmPassword}
              placeholder="Confirm your password"
              autocomplete="new-password"
            />
          {/if}
        </div>

        <div class="password-requirements">
          <p class="text-sm text-muted">Password requirements:</p>
          <ul class="requirements-list">
            <li class:valid={password.length >= 8}>At least 8 characters</li>
            <li class:valid={/[A-Z]/.test(password)}>One uppercase letter</li>
            <li class:valid={/[a-z]/.test(password)}>One lowercase letter</li>
            <li class:valid={/[0-9]/.test(password)}>One number</li>
          </ul>
        </div>
      {/if}

      {#if validationError || error}
        <div class="error-message animate-shake">
          {validationError || error}
        </div>
      {/if}

      <button type="submit" class="btn btn-primary btn-lg w-full">
        {#if isFirstTime}
          Create Vault
        {:else}
          Unlock Vault
        {/if}
      </button>
    </form>

    <p class="security-note">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>
      Your data is encrypted with AES-256-GCM
    </p>
  </div>
</div>

<style>
  .lock-screen {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--bg-primary) 0%, #1a1025 100%);
    padding: var(--spacing-lg);
  }

  .lock-container {
    width: 100%;
    max-width: 400px;
    text-align: center;
  }

  .lock-icon {
    width: 80px;
    height: 80px;
    margin: 0 auto var(--spacing-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--accent-primary) 0%, #9333ea 100%);
    border-radius: var(--radius-xl);
    color: white;
  }

  .lock-title {
    margin-bottom: var(--spacing-sm);
    font-size: 28px;
    font-weight: 600;
  }

  .lock-subtitle {
    color: var(--text-secondary);
    margin-bottom: var(--spacing-xl);
  }

  .lock-form {
    text-align: left;
  }

  .password-input-wrapper {
    position: relative;
  }

  .toggle-password {
    position: absolute;
    right: var(--spacing-md);
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: var(--spacing-xs);
  }

  .toggle-password:hover {
    color: var(--text-primary);
  }

  .password-requirements {
    margin-bottom: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
  }

  .requirements-list {
    list-style: none;
    margin-top: var(--spacing-sm);
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-xs);
  }

  .requirements-list li {
    font-size: 12px;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .requirements-list li::before {
    content: '○';
    font-size: 8px;
  }

  .requirements-list li.valid {
    color: var(--success);
  }

  .requirements-list li.valid::before {
    content: '●';
  }

  .error-message {
    padding: var(--spacing-sm) var(--spacing-md);
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    border-radius: var(--radius-md);
    color: var(--error);
    font-size: 14px;
    margin-bottom: var(--spacing-md);
  }

  .security-note {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-xl);
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
