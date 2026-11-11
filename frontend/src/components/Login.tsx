import { useState, type FormEvent } from "react";
import { api, setToken, type User } from "../api";
import { Logo } from "../App";

export default function Login({ onSignedIn }: { onSignedIn: (u: User) => void }) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const res = await api.login(username.trim(), password);
      setToken(res.access_token);
      onSignedIn(res.user);
    } catch (err) {
      setError((err as Error).message);
      setPassword("");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="login">
      <section className="login-intro">
        <Logo className="login-logo" />
        <h1>Welcome to the SPIL Labs assistant</h1>
        <p>
          Ask about consultations, clients, meetings, orders and feedback in your own words, and get answers straight
          from the company database.
        </p>
        <ul className="login-points">
          <li>Finds records by meaning, not just matching words</li>
          <li>Shows exactly which tool answered each question</li>
          <li>Reports on revenue, workload and client satisfaction</li>
        </ul>
      </section>
      <form className="login-form" onSubmit={submit} noValidate>
        <h2>Sign in</h2>
        <label>
          Username
          <input autoFocus autoComplete="username" value={username} onChange={(e) => setUsername(e.target.value)} required />
        </label>
        <label>
          Password
          <input type="password" autoComplete="current-password" value={password}
                 onChange={(e) => setPassword(e.target.value)} required />
        </label>
        {error && <p className="error" role="alert">{error}</p>}
        <button className="primary" disabled={busy || !username || !password}>{busy ? "Signing in…" : "Sign in"}</button>
        <p className="hint">Ask your administrator if you need an account.</p>
      </form>
    </div>
  );
}
