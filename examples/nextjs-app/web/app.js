import React, {
  useState,
  useEffect,
  createContext,
  useContext,
} from "https://esm.sh/react@18?dev&pin=v135";
import { createRoot } from "https://esm.sh/react-dom@18/client?dev&pin=v135";
import htm from "https://esm.sh/htm@3.1.1?dev&pin=v135";

const html = htm.bind(React.createElement);

// Router Context - Next.js-style client-side routing
const RouterContext = createContext({
  pathname: "/",
  push: () => {},
});

function useRouter() {
  return useContext(RouterContext);
}

// Layout Component - wraps all pages
function Layout({ children }) {
  const router = useRouter();

  const navStyle = {
    display: "flex",
    gap: "1rem",
    padding: "1rem",
    borderBottom: "1px solid #eee",
    background: "#fafafa",
  };

  const linkStyle = (path) => ({
    padding: "0.5rem 1rem",
    textDecoration: "none",
    color: router.pathname === path ? "#0070f3" : "#666",
    fontWeight: router.pathname === path ? "bold" : "normal",
    cursor: "pointer",
  });

  return html`
    <div>
      <nav style=${navStyle}>
        <span style=${linkStyle("/")} onClick=${() => router.push("/")}>Home</span>
        <span style=${linkStyle("/about")} onClick=${() => router.push("/about")}>About</span>
        <span style=${linkStyle("/dashboard")} onClick=${() => router.push("/dashboard")}>Dashboard</span>
      </nav>
      <main style=${{ padding: "2rem" }}>
        ${children}
      </main>
    </div>
  `;
}

function HomePage({ data }) {
  if (!data) return html`<div>Loading...</div>`;
  return html`
    <div>
      <h1 style=${{ marginBottom: "1rem" }}>${data.title}</h1>
      <p>${data.content}</p>
    </div>
  `;
}

function AboutPage({ data }) {
  if (!data) return html`<div>Loading...</div>`;
  return html`
    <div>
      <h1 style=${{ marginBottom: "1rem" }}>${data.title}</h1>
      <p>${data.content}</p>
      <p style=${{ marginTop: "1rem", color: "#666" }}>
        This example shows how to implement Next.js-style patterns in a Forge app,
        including client-side routing and backend data fetching via IPC.
      </p>
    </div>
  `;
}

function DashboardPage({ data }) {
  if (!data) return html`<div>Loading...</div>`;
  return html`
    <div>
      <h1 style=${{ marginBottom: "1rem" }}>${data.title}</h1>
      <div style=${{
        display: "grid",
        gridTemplateColumns: "repeat(3, 1fr)",
        gap: "1rem",
      }}>
        ${data.stats.map(
          (stat, i) =>
            html`<div key=${i} style=${{
              padding: "1.5rem",
              background: "#f5f5f5",
              borderRadius: "8px",
              textAlign: "center",
            }}>
              <div style=${{ fontSize: "2rem", fontWeight: "bold" }}>${stat.value}</div>
              <div style=${{ color: "#666", marginTop: "0.5rem" }}>${stat.label}</div>
            </div>`
        )}
      </div>
    </div>
  `;
}

function NotFoundPage() {
  return html`
    <div>
      <h1>404 - Page Not Found</h1>
      <p>The page you're looking for doesn't exist.</p>
    </div>
  `;
}

// Main App with Router
function App() {
  const [pathname, setPathname] = useState("/");
  const [pageData, setPageData] = useState({});
  const [loading, setLoading] = useState(true);

  const push = (path) => {
    setPathname(path);
    setLoading(true);
    window.host?.send?.("fetch-data", { page: path });
  };

  useEffect(() => {
    const handler = (response) => {
      const { page, data } = response ?? {};
      if (!page) return;
      setPageData((prev) => ({ ...prev, [page]: data }));
      setLoading(false);
    };

    window.host?.on?.("page-data", handler);
    window.host?.send?.("fetch-data", { page: "/" });
  }, []);

  const renderPage = () => {
    const data = pageData[pathname];

    switch (pathname) {
      case "/":
        return html`<${HomePage} data=${data} />`;
      case "/about":
        return html`<${AboutPage} data=${data} />`;
      case "/dashboard":
        return html`<${DashboardPage} data=${data} />`;
      default:
        return html`<${NotFoundPage} />`;
    }
  };

  return html`
    <${RouterContext.Provider} value=${{ pathname, push }}>
      <${Layout}>
        ${loading && !pageData[pathname]
          ? html`<div style=${{ color: "#666" }}>Loading...</div>`
          : renderPage()}
      <//>
    <//>
  `;
}

const rootEl = document.getElementById("root");
if (rootEl) {
  const root = createRoot(rootEl);
  root.render(html`<${App} />`);
}
