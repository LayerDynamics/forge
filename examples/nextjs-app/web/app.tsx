import React, { useState, useEffect, createContext, useContext, ReactNode } from 'react';
import { createRoot } from 'react-dom/client';

declare global {
  interface Window {
    host: {
      send: (channel: string, data?: unknown) => void;
      on: (channel: string, callback: (data: unknown) => void) => void;
    };
  }
}

// Router Context - Next.js-style client-side routing
interface RouterContextType {
  pathname: string;
  push: (path: string) => void;
}

const RouterContext = createContext<RouterContextType>({
  pathname: "/",
  push: () => {}
});

function useRouter() {
  return useContext(RouterContext);
}

// Layout Component - wraps all pages
function Layout({ children }: { children: ReactNode }) {
  const router = useRouter();

  const navStyle = {
    display: 'flex',
    gap: '1rem',
    padding: '1rem',
    borderBottom: '1px solid #eee',
    background: '#fafafa'
  };

  const linkStyle = (path: string) => ({
    padding: '0.5rem 1rem',
    textDecoration: 'none',
    color: router.pathname === path ? '#0070f3' : '#666',
    fontWeight: router.pathname === path ? 'bold' : 'normal',
    cursor: 'pointer'
  });

  return (
    <div>
      <nav style={navStyle}>
        <span style={linkStyle("/")} onClick={() => router.push("/")}>Home</span>
        <span style={linkStyle("/about")} onClick={() => router.push("/about")}>About</span>
        <span style={linkStyle("/dashboard")} onClick={() => router.push("/dashboard")}>Dashboard</span>
      </nav>
      <main style={{ padding: '2rem' }}>
        {children}
      </main>
    </div>
  );
}

// Page Components
function HomePage({ data }: { data: { title: string; content: string } | null }) {
  if (!data) return <div>Loading...</div>;
  return (
    <div>
      <h1 style={{ marginBottom: '1rem' }}>{data.title}</h1>
      <p>{data.content}</p>
    </div>
  );
}

function AboutPage({ data }: { data: { title: string; content: string } | null }) {
  if (!data) return <div>Loading...</div>;
  return (
    <div>
      <h1 style={{ marginBottom: '1rem' }}>{data.title}</h1>
      <p>{data.content}</p>
      <p style={{ marginTop: '1rem', color: '#666' }}>
        This example shows how to implement Next.js-style patterns in a Forge app,
        including client-side routing and backend data fetching via IPC.
      </p>
    </div>
  );
}

function DashboardPage({ data }: { data: { title: string; stats: Array<{ label: string; value: string | number }> } | null }) {
  if (!data) return <div>Loading...</div>;
  return (
    <div>
      <h1 style={{ marginBottom: '1rem' }}>{data.title}</h1>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '1rem' }}>
        {data.stats.map((stat, i) => (
          <div key={i} style={{
            padding: '1.5rem',
            background: '#f5f5f5',
            borderRadius: '8px',
            textAlign: 'center'
          }}>
            <div style={{ fontSize: '2rem', fontWeight: 'bold' }}>{stat.value}</div>
            <div style={{ color: '#666', marginTop: '0.5rem' }}>{stat.label}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

function NotFoundPage() {
  return (
    <div>
      <h1>404 - Page Not Found</h1>
      <p>The page you're looking for doesn't exist.</p>
    </div>
  );
}

// Main App with Router
function App() {
  const [pathname, setPathname] = useState("/");
  const [pageData, setPageData] = useState<Record<string, unknown>>({});
  const [loading, setLoading] = useState(true);

  const push = (path: string) => {
    setPathname(path);
    setLoading(true);
    window.host.send("fetch-data", { page: path });
  };

  useEffect(() => {
    // Listen for page data from backend
    window.host.on("page-data", (response: unknown) => {
      const { page, data } = response as { page: string; data: unknown };
      setPageData(prev => ({ ...prev, [page]: data }));
      setLoading(false);
    });

    // Initial data fetch
    window.host.send("fetch-data", { page: "/" });
  }, []);

  const renderPage = () => {
    const data = pageData[pathname];

    switch (pathname) {
      case "/":
        return <HomePage data={data as { title: string; content: string } | null} />;
      case "/about":
        return <AboutPage data={data as { title: string; content: string } | null} />;
      case "/dashboard":
        return <DashboardPage data={data as { title: string; stats: Array<{ label: string; value: string | number }> } | null} />;
      default:
        return <NotFoundPage />;
    }
  };

  return (
    <RouterContext.Provider value={{ pathname, push }}>
      <Layout>
        {loading && !pageData[pathname] ? (
          <div style={{ color: '#666' }}>Loading...</div>
        ) : (
          renderPage()
        )}
      </Layout>
    </RouterContext.Provider>
  );
}

const root = createRoot(document.getElementById('root')!);
root.render(<App />);
