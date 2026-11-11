import { useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { api, type ChatResponse, type Mode, type SearchResult, type Source, type Status, type User } from "../api";
import { Logo } from "../App";
import { DataTable, highlight, label, ScoreGauge, Value } from "./format";
import RecordDrawer from "./RecordDrawer";

export interface ChatMsg {
  id: string;
  role: "user" | "assistant";
  text: string;
  response?: ChatResponse;
  error?: boolean;
  pending?: boolean;
}

export interface Conversation {
  id: string;
  title: string;
  createdAt: string;
  messages: ChatMsg[];
}

// crypto.randomUUID only exists on https/localhost; the app may be opened over plain http on a LAN.
const makeId = () => Date.now().toString(36) + Math.random().toString(36).slice(2, 8);

export const newConversation = (): Conversation =>
  ({ id: makeId(), title: "New chat", createdAt: new Date().toISOString(), messages: [] });

const ERP_SOURCES = "consultations,meetings,feedback,orders";

const STARTERS = [
  { kind: "Look up", text: "Which consultations are in progress?" },
  { kind: "Search", text: "Unhappy clients" },
  { kind: "Report", text: "Revenue by month" },
  { kind: "Look up", text: "Show upcoming meetings" },
];

function greeting() {
  const h = new Date().getHours();
  return h < 12 ? "Good morning" : h < 17 ? "Good afternoon" : "Good evening";
}

export default function ChatPage({ user, status, conversation, onChange }: {
  user: User;
  status: Status | null;
  conversation: Conversation;
  onChange: (c: Conversation) => void;
}) {
  const [text, setText] = useState("");
  const [source, setSource] = useState("");
  const [mode, setMode] = useState<Mode>("semantic");
  const [sources, setSources] = useState<Source[]>([]);
  const [open, setOpen] = useState<SearchResult | null>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const ready = !!status?.index;
  const busy = conversation.messages.some((m) => m.pending);
  const empty = conversation.messages.length === 0;
  const threshold = status?.index?.recommended_min_score ?? 0.25;

  useEffect(() => {
    if (ready) api.sources().then((r) => setSources(r.sources)).catch(() => {});
  }, [ready]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [conversation.messages.length, conversation.messages[conversation.messages.length - 1]?.pending]);

  useEffect(() => { inputRef.current?.focus(); }, [conversation.id]);

  async function send(message: string) {
    const q = message.trim();
    if (!q || busy || !ready) return;
    setText("");

    const userMsg: ChatMsg = { id: makeId(), role: "user", text: q };
    const pending: ChatMsg = { id: makeId(), role: "assistant", text: "", pending: true };
    const title = conversation.messages.length ? conversation.title : q.length > 48 ? q.slice(0, 47) + "…" : q;
    const withPending: Conversation = { ...conversation, title, messages: [...conversation.messages, userMsg, pending] };
    onChange(withPending);

    let reply: ChatMsg;
    try {
      const res = await api.chat({ message: q, source: source || null, mode });
      reply = { id: pending.id, role: "assistant", text: res.reply, response: res };
    } catch (err) {
      reply = { id: pending.id, role: "assistant", text: (err as Error).message, error: true };
    }
    onChange({ ...withPending, messages: [...withPending.messages.slice(0, -1), reply] });
  }

  function submit(e: FormEvent) { e.preventDefault(); send(text); }

  function onKey(e: KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) { e.preventDefault(); send(text); }
  }

  const sourceName = (n: string) => sources.find((s) => s.name === n)?.display_name ?? n;

  const composer = (
    <form className="composer" onSubmit={submit}>
      <label htmlFor="ask" className="sr">Your question</label>
      <textarea
        id="ask" ref={inputRef} rows={1} value={text} maxLength={500}
        onChange={(e) => { setText(e.target.value); e.target.style.height = "auto"; e.target.style.height = `${Math.min(e.target.scrollHeight, 200)}px`; }}
        onKeyDown={onKey}
        className={mode === "pattern" ? "mono" : ""}
        placeholder={mode === "pattern" ? "Type a regular expression, e.g. (refund|money back)" : "Ask about consultations, clients, orders or products…"}
      />
      <div className="composer-bar">
        <label className="pick">
          <span className="sr">Search in</span>
          <select value={source} onChange={(e) => setSource(e.target.value)} title="Where meaning searches look">
            <option value="">Everything</option>
            <option value={ERP_SOURCES}>All ERP records</option>
            {sources.map((s) => <option key={s.name} value={s.name}>{s.display_name}</option>)}
          </select>
        </label>
        <label className="pick">
          <span className="sr">Match by</span>
          <select value={mode} onChange={(e) => setMode(e.target.value as Mode)} title="How searches match">
            <option value="semantic">Meaning</option>
            <option value="keyword">Words only</option>
            <option value="pattern">Regex pattern</option>
          </select>
        </label>
        <span className="composer-hint">Enter to send · Shift+Enter for a new line</span>
        <button className="send" aria-label="Send" disabled={!text.trim() || busy || !ready}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 19V5M5.5 11.5L12 5l6.5 6.5" /></svg>
        </button>
      </div>
    </form>
  );

  return (
    <div className={`chat ${empty ? "is-empty" : ""}`}>
      {empty ? (
        <section className="welcome">
          <Logo className="welcome-logo" />
          <h1>{greeting()}, {user.username.charAt(0).toUpperCase() + user.username.slice(1)}</h1>
          <p className="welcome-sub">
            Ask about ERP consultations, clients, meetings, orders, feedback or products. Questions are answered from
            your SQL Server data through the MCP server.
          </p>
          {!ready && <p className="notice">{status?.error ? `The data server isn't running: ${status.error.split("\n")[0]}` : "Connecting to the data server…"}</p>}
          {composer}
          <ul className="starters">
            {STARTERS.map((s) => (
              <li key={s.text}>
                <button disabled={!ready} onClick={() => send(s.text)}>
                  <span className="starter-kind">{s.kind}</span>
                  {s.text}
                </button>
              </li>
            ))}
          </ul>
        </section>
      ) : (
        <>
          <div className="thread" role="log" aria-live="polite">
            {conversation.messages.map((m) => m.role === "user" ? (
              <div key={m.id} className="msg user"><p>{m.text}</p></div>
            ) : (
              <AssistantMessage key={m.id} msg={m} threshold={threshold} sourceName={sourceName}
                                onOpen={setOpen} onAsk={send} />
            ))}
            <div ref={endRef} />
          </div>
          <div className="dock">
            {composer}
            <p className="dock-note">Answers come straight from the database. Nothing is invented; check the tool used under each reply.</p>
          </div>
        </>
      )}
      {open && <RecordDrawer result={open} sourceName={sourceName(open.source)} onClose={() => setOpen(null)} />}
    </div>
  );
}

function AssistantMessage({ msg, threshold, sourceName, onOpen, onAsk }: {
  msg: ChatMsg; threshold: number; sourceName: (n: string) => string;
  onOpen: (r: SearchResult) => void; onAsk: (q: string) => void;
}) {
  if (msg.pending) {
    return (
      <div className="msg assistant">
        <Avatar />
        <div className="msg-body"><span className="typing" aria-label="Working"><i /><i /><i /></span></div>
      </div>
    );
  }
  const r = msg.response;
  return (
    <div className="msg assistant">
      <Avatar />
      <div className="msg-body">
        <p className={msg.error ? "error" : "reply"}>{msg.text}</p>
        {r && <ToolChip r={r} />}
        {r && <ReplyContent r={r} threshold={threshold} sourceName={sourceName} onOpen={onOpen} onAsk={onAsk} />}
      </div>
    </div>
  );
}

function Avatar() {
  return <span className="bot-avatar" aria-hidden="true"><img src="/spil-labs-mark.png" alt="" /></span>;
}

/** Shows which MCP tool answered, with its arguments, so replies are never a black box. */
function ToolChip({ r }: { r: ChatResponse }) {
  const args = Object.entries(r.arguments)
    .filter(([k, v]) => k !== "query" && v !== null && v !== undefined && v !== "")
    .map(([k, v]) => `${label(k).toLowerCase()}: ${String(v)}`);
  return (
    <p className="tool-chip" title="The MCP tool the server ran for this answer">
      <span className="tool-chip-name">{r.tool}</span>
      {args.length > 0 && <span>{args.join(" · ")}</span>}
      <span>{r.took_ms} ms</span>
    </p>
  );
}

function ReplyContent({ r, threshold, sourceName, onOpen, onAsk }: {
  r: ChatResponse; threshold: number; sourceName: (n: string) => string;
  onOpen: (res: SearchResult) => void; onAsk: (q: string) => void;
}) {
  if (r.kind === "search") {
    const results: SearchResult[] = r.data?.results ?? [];
    if (!results.length) return null;
    return (
      <ol className="answers">
        {results.map((x) => (
          <li key={`${x.source}-${x.id}`}>
            <button className="answer" onClick={() => onOpen(x)}>
              <div className="answer-main">
                <span className="tag">{sourceName(x.source)}</span>
                <h4>{x.title}</h4>
                <p className="snippet">{highlight(x.snippet, x.matched_words)}</p>
                {(x.matched_concepts.length > 0 || x.matched_words.length > 0) && (
                  <p className="why-chips">
                    {x.matched_concepts.map((c) => <span key={c} className="by meaning">{c}</span>)}
                    {x.matched_words.slice(0, 4).map((w) => <span key={w} className="by keyword">{w}</span>)}
                  </p>
                )}
              </div>
              {r.data?.mode !== "pattern" && <ScoreGauge score={x.score} threshold={threshold} />}
            </button>
          </li>
        ))}
      </ol>
    );
  }

  if (r.kind === "rows") {
    const rows: Record<string, unknown>[] = r.data?.[r.rows_key ?? ""] ?? [];
    if (!rows.length) return null;
    const linkable = rows.some((x) => "consultation_id" in x) || r.rows_key === "consultations";
    const open = (row: Record<string, unknown>) => {
      const id = r.rows_key === "consultations" ? row.id : row.consultation_id;
      if (id !== null && id !== undefined) onAsk(`consultation ${id}`);
    };
    return (
      <div className="reply-block">
        <DataTable rows={rows} onRowClick={linkable ? open : undefined} />
        {linkable && <p className="hint">Select a row to see the whole consultation.</p>}
      </div>
    );
  }

  return <div className="reply-block"><Value value={r.data} /></div>;
}
