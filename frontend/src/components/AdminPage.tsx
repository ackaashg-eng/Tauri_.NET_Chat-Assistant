import { useEffect, useState, type FormEvent } from "react";
import { api, type Explanation, type McpInfo, type Role, type Status, type User } from "../api";

export default function AdminPage({ me, onIndexChanged }: { me: User; onIndexChanged: () => void }) {
  return (
    <div className="admin">
      <UsersPanel me={me} />
      <ServerPanel onIndexChanged={onIndexChanged} />
    </div>
  );
}

function UsersPanel({ me }: { me: User }) {
  const [users, setUsers] = useState<User[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [role, setRole] = useState<Role>("user");

  const load = () => api.users().then(setUsers).catch((e) => setError(e.message));
  useEffect(() => { load(); }, []);

  async function act(fn: () => Promise<unknown>, done: string) {
    setError(null); setNotice(null);
    try { await fn(); setNotice(done); await load(); } catch (e) { setError((e as Error).message); }
  }

  async function create(e: FormEvent) {
    e.preventDefault();
    await act(() => api.createUser(username.trim(), password, role), `Added ${username.trim()}.`);
    setUsername(""); setPassword(""); setRole("user");
  }

  function resetPassword(u: User) {
    const pw = window.prompt(`New password for ${u.username} (at least 10 characters, letters and numbers):`);
    if (pw) act(() => api.resetPassword(u.id, pw), `Password changed for ${u.username}. They have been signed out.`);
  }

  function remove(u: User) {
    if (window.confirm(`Delete ${u.username}? This can't be undone.`)) act(() => api.deleteUser(u.id), `Deleted ${u.username}.`);
  }

  return (
    <section className="panel">
      <h2>People with access</h2>
      <p className="muted">Admins can manage people and rebuild the search index. Users can search.</p>

      {error && <p className="error" role="alert">{error}</p>}
      {notice && <p className="ok" role="status">{notice}</p>}

      <table className="users">
        <thead><tr><th>Username</th><th>Role</th><th>Status</th><th><span className="sr">Actions</span></th></tr></thead>
        <tbody>
          {users.map((u) => (
            <tr key={u.id} className={u.active ? "" : "inactive"}>
              <td>{u.username}{u.id === me.id && <span className="you">you</span>}</td>
              <td>
                <select value={u.role} disabled={u.id === me.id} aria-label={`Role for ${u.username}`}
                        onChange={(e) => act(() => api.updateUser(u.id, { role: e.target.value as Role }), `${u.username} is now ${e.target.value === "admin" ? "an admin" : "a user"}.`)}>
                  <option value="user">User</option>
                  <option value="admin">Admin</option>
                </select>
              </td>
              <td>{u.active ? "Active" : "Disabled"}</td>
              <td>
                {u.id !== me.id && (
                  <div className="row-actions">
                    <button className="link" onClick={() => act(() => api.updateUser(u.id, { active: !u.active }), `${u.username} ${u.active ? "disabled" : "enabled"}.`)}>
                      {u.active ? "Disable" : "Enable"}
                    </button>
                    <button className="link" onClick={() => resetPassword(u)}>Set password</button>
                    <button className="link danger" onClick={() => remove(u)}>Delete</button>
                  </div>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <form className="add-user" onSubmit={create}>
        <h3>Add a person</h3>
        <label>Username<input value={username} onChange={(e) => setUsername(e.target.value)} pattern="[A-Za-z0-9_.\-]{3,32}" required /></label>
        <label>Temporary password<input type="password" autoComplete="new-password" value={password} onChange={(e) => setPassword(e.target.value)} required /></label>
        <label>Role
          <select value={role} onChange={(e) => setRole(e.target.value as Role)}>
            <option value="user">User</option><option value="admin">Admin</option>
          </select>
        </label>
        <button className="primary" disabled={!username || !password}>Add person</button>
      </form>
    </section>
  );
}

function ServerPanel({ onIndexChanged }: { onIndexChanged: () => void }) {
  const [info, setInfo] = useState<McpInfo | null>(null);
  const [status, setStatus] = useState<Status | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<Record<string, unknown> | null>(null);
  const [openTool, setOpenTool] = useState<string | null>(null);
  const [probe, setProbe] = useState("");
  const [explained, setExplained] = useState<Explanation | null>(null);

  useEffect(() => {
    api.mcpInfo().then(setInfo).catch((e) => setError(e.message));
    api.status().then(setStatus).catch(() => {});
  }, []);

  async function reindex() {
    setBusy(true); setError(null); setReport(null);
    try { setReport(await api.reindex()); onIndexChanged(); api.status().then(setStatus); }
    catch (e) { setError((e as Error).message); }
    finally { setBusy(false); }
  }

  async function explain(e: FormEvent) {
    e.preventDefault();
    if (!probe.trim()) return;
    try { setExplained(await api.explain(probe.trim())); } catch (err) { setError((err as Error).message); }
  }

  const server = info?.server?.serverInfo;
  const idx = status?.index;
  return (
    <section className="panel">
      <h2>Search server</h2>
      <p className="muted">
        The backend starts the C# MCP server as a child process and talks to it over standard input and output.
        The server reads Microsoft SQL Server and matches text with regular-expression rules. No AI model is used.
      </p>

      {error && <p className="error" role="alert">{error}</p>}
      <dl className="kv">
        {info && <div><dt>MCP server</dt><dd>{server?.name} {server?.version}</dd></div>}
        {info && <div><dt>MCP protocol</dt><dd>{info.server?.protocolVersion}</dd></div>}
        {idx && <div><dt>Database</dt><dd>{idx.database}</dd></div>}
        {idx && <div><dt>Search index</dt><dd>{idx.records} records, {idx.concepts} meaning rules, {idx.vocabulary} words</dd></div>}
        {idx && <div className="wide"><dt>Rules file</dt><dd className="path">{idx.rules_file}</dd></div>}
      </dl>

      <h3>Reload rules and data</h3>
      <p className="muted">Run this after changing rows in SQL Server or editing the rules file. No restart is needed.</p>
      <div className="dialog-actions left">
        <button className="primary" disabled={busy} onClick={reindex}>{busy ? "Reloading…" : "Reload rules and data"}</button>
      </div>
      {report && (
        <p className="ok" role="status">
          Reloaded {String(report.records)} records and {String(report.concepts)} meaning rules in {String(report.took_ms)} ms.
        </p>
      )}

      <h3>Test a question against the rules</h3>
      <p className="muted">See which words are compared and which meaning rules fire, with the regex that matched.</p>
      <form className="probe" onSubmit={explain}>
        <input value={probe} onChange={(e) => setProbe(e.target.value)} placeholder="e.g. where is my package" aria-label="Question to test" />
        <button className="ghost">Test</button>
      </form>
      {explained && (
        <div className="explained">
          <p>
            Words compared:{" "}
            {explained.words.length ? explained.words.map((w) => (
              <span key={w.word} className="by keyword" title={`compared as "${w.compared_as}"`}>
                {w.word}{w.compared_as !== w.word ? ` → ${w.compared_as}` : ""}
              </span>
            )) : "none"}
          </p>
          {explained.concepts.length ? (
            <ul>
              {explained.concepts.map((c) => (
                <li key={c.concept}><span className="concept">{c.concept}</span> <code>{c.matched_pattern}</code></li>
              ))}
            </ul>
          ) : <p className="muted">No meaning rule matched. Add a pattern to the rules file to teach it this phrase.</p>}
        </div>
      )}

      <h3>Tools this server offers</h3>
      <p className="muted">Discovered with the MCP <code>tools/list</code> request.</p>
      <ul className="tools">
        {info?.tools.map((t) => (
          <li key={t.name}>
            <button className="tool" aria-expanded={openTool === t.name} onClick={() => setOpenTool(openTool === t.name ? null : t.name)}>
              <code>{t.name}</code>
              {t.read_only !== null && <span>{t.read_only ? "Read-only" : "Changes data"}</span>}
            </button>
            <p>{t.description}</p>
            {openTool === t.name && <pre>{JSON.stringify(t.input_schema, null, 2)}</pre>}
          </li>
        ))}
      </ul>
    </section>
  );
}
