import { createApp, ref } from 'https://unpkg.com/vue@3/dist/vue.esm-browser.js';

const App = {
  setup() {
    const count = ref(0);
    const increment = () => count.value++;
    return { count, increment };
  },
  template: `
    <div style="font-family: system-ui; padding: 2rem;">
      <h1>Forge Vue App</h1>
      <p>Count: {{ count }}</p>
      <button @click="increment">Increment</button>
      <p style="margin-top: 1rem; color: #666;">
        Edit web/main.js to get started
      </p>
    </div>
  `
};

createApp(App).mount('#app');
