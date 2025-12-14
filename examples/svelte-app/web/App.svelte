<script lang="ts">
  import { onMount } from 'svelte';

  interface Todo {
    id: number;
    text: string;
    done: boolean;
  }

  let todos: Todo[] = [];
  let newTodoText = '';
  let status = '';

  declare global {
    interface Window {
      host: {
        send: (channel: string, data?: unknown) => void;
        on: (channel: string, callback: (data: unknown) => void) => void;
      };
    }
  }

  onMount(() => {
    // Listen for todos from backend
    window.host.on('todos-loaded', (data: unknown) => {
      const { todos: loadedTodos } = data as { todos: Todo[] };
      todos = loadedTodos;
      status = 'Todos loaded from backend';
    });

    window.host.on('todos-saved', (data: unknown) => {
      const { success } = data as { success: boolean };
      status = success ? 'Todos saved!' : 'Failed to save todos';
    });

    // Request initial todos
    window.host.send('get-todos');
  });

  function addTodo() {
    if (!newTodoText.trim()) return;

    const newTodo: Todo = {
      id: Date.now(),
      text: newTodoText.trim(),
      done: false
    };
    todos = [...todos, newTodo];
    newTodoText = '';
    saveTodos();
  }

  function toggleTodo(id: number) {
    todos = todos.map(todo =>
      todo.id === id ? { ...todo, done: !todo.done } : todo
    );
    saveTodos();
  }

  function deleteTodo(id: number) {
    todos = todos.filter(todo => todo.id !== id);
    saveTodos();
  }

  function saveTodos() {
    window.host.send('save-todos', { todos });
  }

  $: remaining = todos.filter(t => !t.done).length;
</script>

<div class="container">
  <h1>Svelte Todo App</h1>

  <form on:submit|preventDefault={addTodo} class="add-form">
    <input
      type="text"
      bind:value={newTodoText}
      placeholder="Add a new todo..."
      class="input"
    />
    <button type="submit" class="btn">Add</button>
  </form>

  <ul class="todo-list">
    {#each todos as todo (todo.id)}
      <li class="todo-item" class:done={todo.done}>
        <label class="checkbox-label">
          <input
            type="checkbox"
            checked={todo.done}
            on:change={() => toggleTodo(todo.id)}
          />
          <span class="todo-text">{todo.text}</span>
        </label>
        <button class="delete-btn" on:click={() => deleteTodo(todo.id)}>
          Delete
        </button>
      </li>
    {/each}
  </ul>

  {#if todos.length > 0}
    <p class="status">
      {remaining} item{remaining === 1 ? '' : 's'} remaining
    </p>
  {/if}

  {#if status}
    <p class="notification">{status}</p>
  {/if}

  <p class="hint">Edit web/App.svelte to customize this app</p>
</div>

<style>
  .container {
    padding: 2rem;
    max-width: 500px;
    margin: 0 auto;
  }

  h1 {
    margin-bottom: 1.5rem;
    color: #333;
  }

  .add-form {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }

  .input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    background: #ff3e00;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
  }

  .btn:hover {
    background: #e63600;
  }

  .todo-list {
    list-style: none;
    margin-bottom: 1rem;
  }

  .todo-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem;
    border-bottom: 1px solid #eee;
  }

  .todo-item.done .todo-text {
    text-decoration: line-through;
    color: #999;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .delete-btn {
    padding: 0.25rem 0.5rem;
    background: transparent;
    border: 1px solid #ddd;
    border-radius: 4px;
    color: #666;
    cursor: pointer;
  }

  .delete-btn:hover {
    background: #fee;
    border-color: #fcc;
    color: #c00;
  }

  .status {
    color: #666;
    font-size: 0.9rem;
    margin-bottom: 1rem;
  }

  .notification {
    padding: 0.5rem 1rem;
    background: #d4edda;
    color: #155724;
    border-radius: 4px;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }

  .hint {
    color: #999;
    font-size: 0.9rem;
    margin-top: 2rem;
  }
</style>
