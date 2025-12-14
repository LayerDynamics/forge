import React, { useState, useEffect } from 'react';
import { createRoot } from 'react-dom/client';

declare global {
  interface Window {
    host: {
      send: (channel: string, data?: unknown) => void;
      on: (channel: string, callback: (data: unknown) => void) => void;
    };
  }
}

function App() {
  const [count, setCount] = useState(0);
  const [messages, setMessages] = useState<string[]>([]);

  useEffect(() => {
    // Listen for messages from the Deno backend
    window.host.on("pong", (data: unknown) => {
      const { timestamp } = data as { timestamp: number };
      setMessages(prev => [...prev, `Pong received at ${new Date(timestamp).toLocaleTimeString()}`]);
    });
  }, []);

  const sendPing = () => {
    window.host.send("ping", { count });
    setMessages(prev => [...prev, `Ping sent with count: ${count}`]);
  };

  return (
    <div style={{ padding: '2rem', maxWidth: '600px', margin: '0 auto' }}>
      <h1 style={{ marginBottom: '1rem' }}>React App</h1>

      <section style={{ marginBottom: '2rem' }}>
        <h2 style={{ fontSize: '1.2rem', marginBottom: '0.5rem' }}>Counter</h2>
        <p style={{ marginBottom: '0.5rem' }}>Count: {count}</p>
        <button
          onClick={() => setCount(c => c + 1)}
          style={{ padding: '0.5rem 1rem', marginRight: '0.5rem' }}
        >
          Increment
        </button>
        <button
          onClick={() => setCount(0)}
          style={{ padding: '0.5rem 1rem' }}
        >
          Reset
        </button>
      </section>

      <section style={{ marginBottom: '2rem' }}>
        <h2 style={{ fontSize: '1.2rem', marginBottom: '0.5rem' }}>IPC Demo</h2>
        <button
          onClick={sendPing}
          style={{ padding: '0.5rem 1rem' }}
        >
          Send Ping to Backend
        </button>
        <div style={{
          marginTop: '1rem',
          padding: '1rem',
          background: '#f5f5f5',
          borderRadius: '4px',
          maxHeight: '200px',
          overflow: 'auto'
        }}>
          {messages.length === 0 ? (
            <p style={{ color: '#666' }}>No messages yet</p>
          ) : (
            messages.map((msg, i) => (
              <p key={i} style={{ fontSize: '0.9rem', marginBottom: '0.25rem' }}>{msg}</p>
            ))
          )}
        </div>
      </section>

      <p style={{ color: '#666', fontSize: '0.9rem' }}>
        Edit web/main.tsx to customize this app
      </p>
    </div>
  );
}

const root = createRoot(document.getElementById('root')!);
root.render(<App />);
