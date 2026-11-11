import type { Status, User } from "../api";
import { Logo, type View } from "../App";
import type { Conversation } from "./ChatPage";

interface Props {
  user: User;
  status: Status | null;
  view: View;
  chats: Conversation[];
  activeId: string | null;
  onNewChat: () => void;
  onOpenChat: (id: string) => void;
  onDeleteChat: (id: string) => void;
  onNavigate: (v: View) => void;
  onAccount: () => void;
  onSignOut: () => void;
}

export default function Sidebar(p: Props) {
  return (
    <aside className="sidebar" aria-label="Navigation">
      <div className="side-brand">
        <Logo />
      </div>

      <button className="new-chat" onClick={p.onNewChat}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
        New chat
      </button>

      <nav className="side-nav" aria-label="Pages">
        <button className={p.view === "tools" ? "on" : ""} onClick={() => p.onNavigate("tools")}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 6h16M4 12h10M4 18h7" /></svg>
          Tools
        </button>
        {p.user.role === "admin" && (
          <button className={p.view === "admin" ? "on" : ""} onClick={() => p.onNavigate("admin")}>
            <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="8" r="3.5" /><path d="M5 20c1.2-3.6 4-5.5 7-5.5s5.8 1.9 7 5.5" /></svg>
            Admin
          </button>
        )}
      </nav>

      <div className="recents">
        <h2>Recent chats</h2>
        {p.chats.length === 0 ? (
          <p className="recents-empty">Your conversations will appear here.</p>
        ) : (
          <ul>
            {p.chats.map((c) => (
              <li key={c.id} className={p.view === "chat" && c.id === p.activeId ? "on" : ""}>
                <button className="recent" onClick={() => p.onOpenChat(c.id)} title={c.title}>{c.title}</button>
                <button className="recent-delete" aria-label={`Delete chat “${c.title}”`} onClick={() => p.onDeleteChat(c.id)}>
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="side-foot">
        <ServerState status={p.status} />
        <div className="who">
          <span className="avatar" aria-hidden="true">{p.user.username.slice(0, 1).toUpperCase()}</span>
          <span className="who-name">
            <strong>{p.user.username}</strong>
            <small>{p.user.role === "admin" ? "Administrator" : "Member"}</small>
          </span>
          <span className="who-actions">
            <button className="link small" onClick={p.onAccount}>Change password</button>
            <button className="link small" onClick={p.onSignOut}>Sign out</button>
          </span>
        </div>
      </div>
    </aside>
  );
}

function ServerState({ status }: { status: Status | null }) {
  let state = "starting", text = "Connecting to the data server…";
  if (status?.index) {
    state = "ok";
    text = `${status.index.records} records · ${status.index.concepts} meaning rules`;
  } else if (status?.error) {
    state = "down";
    text = "Data server unavailable";
  }
  return (
    <p className={`server-state ${state}`} role="status" title={status?.error ?? status?.index?.database ?? ""}>
      <span className="dot" aria-hidden="true" />{text}
    </p>
  );
}
