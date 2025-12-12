import React from 'react';
import { createRoot } from 'react-dom/client';

function App() {
  const [count, setCount] = React.useState(0);

  return (
    <div style={{ fontFamily: 'system-ui', padding: '2rem' }}>
      <h1>Forge React App</h1>
      <p>Count: {count}</p>
      <button onClick={() => setCount(c => c + 1)}>Increment</button>
      <p style={{ marginTop: '1rem', color: '#666' }}>
        Edit web/main.tsx to get started
      </p>
    </div>
  );
}

const root = createRoot(document.getElementById('root')!);
root.render(<App />);
