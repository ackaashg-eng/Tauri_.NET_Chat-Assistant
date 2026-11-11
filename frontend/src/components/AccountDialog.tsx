import { useState, type FormEvent } from "react";
import { api } from "../api";

export default function AccountDialog({ onClose, onPasswordChanged }: { onClose: () => void; onPasswordChanged: () => void }) {
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    if (next !== confirm) { setError("The new passwords don't match."); return; }
    setBusy(true); setError(null);
    try {
      await api.changePassword(current, next);
      onPasswordChanged(); // tokens are invalidated, so sign in again
    } catch (err) {
      setError((err as Error).message);
    } finally { setBusy(false); }
  }

  return (
    <div className="drawer-backdrop center" onClick={onClose}>
      <form className="dialog" role="dialog" aria-modal="true" aria-labelledby="acct-title"
            onClick={(e) => e.stopPropagation()} onSubmit={submit}>
        <h2 id="acct-title">Change your password</h2>
        <label>Current password<input type="password" autoComplete="current-password" value={current} onChange={(e) => setCurrent(e.target.value)} autoFocus /></label>
        <label>New password<input type="password" autoComplete="new-password" value={next} onChange={(e) => setNext(e.target.value)} /></label>
        <label>Repeat new password<input type="password" autoComplete="new-password" value={confirm} onChange={(e) => setConfirm(e.target.value)} /></label>
        <p className="hint">At least 10 characters, mixing letters with numbers or symbols. You'll be asked to sign in again.</p>
        {error && <p className="error" role="alert">{error}</p>}
        <div className="dialog-actions">
          <button type="button" className="ghost" onClick={onClose}>Cancel</button>
          <button className="primary" disabled={busy || !current || !next}>Change password</button>
        </div>
      </form>
    </div>
  );
}
