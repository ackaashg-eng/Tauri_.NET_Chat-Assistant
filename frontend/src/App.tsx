import { useCallback, useEffect, useState } from "react";
import { api, hasToken, setToken, setUnauthorizedHandler, type Status, type User } from "./api";
import Login from "./components/Login";
import ChatPage, { type Conversation, newConversation } from "./components/ChatPage";
import AdminPage from "./components/AdminPage";
import AccountDialog from "./components/AccountDialog";
import ToolsPage from "./components/ToolsPage";
import Sidebar from "./components/Sidebar";

export type View = "chat" | "tools" | "admin";

// Conversations are kept for this browser tab only (cleared when the tab closes).
const CHATS_KEY = "spil.chats";
function loadChats(user: string): Conversation[] {
  try {
    const raw = sessionStorage.getItem(`${CHATS_KEY}.${user}`);
    return raw ? (JSON.parse(raw) as Conversation[]) : [];
  } catch { return []; }
}
function saveChats(user: string, chats: Conversation[]) {
  try { sessionStorage.setItem(`${CHATS_KEY}.${user}`, JSON.stringify(chats.slice(0, 20))); } catch { /* storage full or blocked */ }
}

export default function App() {
  const [user, setUser] = useState<User | null>(null);
  const [checking, setChecking] = useState(hasToken());
  const [view, setView] = useState<View>("chat");
  const [status, setStatus] = useState<Status | null>(null);
  const [accountOpen, setAccountOpen] = useState(false);
  const [chats, setChats] = useState<Conversation[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const [draft, setDraft] = useState<Conversation>(newConversation);

  const signOut = useCallback(() => { setToken(null); setUser(null); setView("chat"); setChats([]); setActiveId(null); }, []);

  useEffect(() => {
    setUnauthorizedHandler(() => setUser(null));
    if (!hasToken()) return;
    api.me().then(setUser).catch(() => setToken(null)).finally(() => setChecking(false));
  }, []);

  useEffect(() => { if (user) setChats(loadChats(user.username)); }, [user]);
  useEffect(() => { if (user) saveChats(user.username, chats); }, [user, chats]);

  // Poll the MCP server status (it can take a moment to start and build its index).
  useEffect(() => {
    if (!user) return;
    let alive = true;
    const load = () => api.status().then((s) => alive && setStatus(s)).catch(() => {});
    load();
    const t = setInterval(load, status?.index ? 30000 : 3000);
    return () => { alive = false; clearInterval(t); };
  }, [user, !!status?.index]);

  const updateChat = useCallback((c: Conversation) => {
    setChats((all) => [c, ...all.filter((x) => x.id !== c.id)]);
    setActiveId(c.id);
  }, []);

  if (checking) return <div className="boot" aria-busy="true" />;
  if (!user) return <Login onSignedIn={(u) => { setUser(u); setStatus(null); }} />;

  const active = chats.find((c) => c.id === activeId) ?? null;
  const go = (v: View) => { setView(v); setMenuOpen(false); };

  return (
    <div className={`shell ${menuOpen ? "menu-open" : ""}`}>
      <Sidebar
        user={user}
        status={status}
        view={view}
        chats={chats}
        activeId={activeId}
        onNewChat={() => { setActiveId(null); setDraft(newConversation()); go("chat"); }}
        onOpenChat={(id) => { setActiveId(id); go("chat"); }}
        onDeleteChat={(id) => { setChats((all) => all.filter((c) => c.id !== id)); if (id === activeId) setActiveId(null); }}
        onNavigate={go}
        onAccount={() => setAccountOpen(true)}
        onSignOut={signOut}
      />
      <button className="menu-button" aria-label="Open menu" onClick={() => setMenuOpen(!menuOpen)}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16M4 12h16M4 17h16" /></svg>
      </button>
      <div className="scrim" onClick={() => setMenuOpen(false)} />

      <main className={`main view-${view}`}>
        {view === "chat" && (
          <ChatPage
            user={user}
            status={status}
            conversation={active ?? draft}
            onChange={updateChat}
          />
        )}
        {view === "tools" && <div className="page"><ToolsPage ready={!!status?.index} /></div>}
        {view === "admin" && <div className="page"><AdminPage me={user} onIndexChanged={() => api.status().then(setStatus)} /></div>}
      </main>

      {accountOpen && <AccountDialog onClose={() => setAccountOpen(false)} onPasswordChanged={signOut} />}
    </div>
  );
}

export function Logo({ className = "" }: { className?: string }) {
  return <img className={`logo ${className}`} src="/spil-labs-logo.png" alt="SPIL Labs" />;
}
