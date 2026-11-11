import { useEffect, useRef, useState } from "react";
import { api, type RecordDetail, type SearchResult } from "../api";
import { formatValue, highlight, label } from "./format";

/** Side panel with the full database row, fetched with the MCP `get_record` tool. */
export default function RecordDrawer({ result, sourceName, onClose }: {
  result: SearchResult; sourceName: string; onClose: () => void;
}) {
  const [record, setRecord] = useState<RecordDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    closeRef.current?.focus();
    api.record(result.source, result.id).then(setRecord).catch((e) => setError(e.message));
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [result, onClose]);

  return (
    <div className="drawer-backdrop" onClick={onClose}>
      <aside className="drawer" role="dialog" aria-modal="true" aria-labelledby="drawer-title" onClick={(e) => e.stopPropagation()}>
        <header>
          <span className={`tag ${result.source}`}>{sourceName} #{result.id}</span>
          <button ref={closeRef} className="link" onClick={onClose}>Close</button>
        </header>
        <h2 id="drawer-title">{result.title}</h2>

        <div className="why">
          <h3>Why this matched</h3>
          {result.matched_by.includes("pattern") ? (
            <p>The pattern matched: {result.matched_words.map((w) => <code key={w} className="hit">{w}</code>)}</p>
          ) : (
            <>
              <p>Match score <strong>{result.score.toFixed(2)}</strong>, from regex rules (no AI model).</p>
              {result.matched_concepts.length > 0 && (
                <p>Same meaning: {result.matched_concepts.map((c) => <span key={c} className="concept">{c}</span>)}</p>
              )}
              {result.matched_words.length > 0 && (
                <p>Shared words: {result.matched_words.map((w) => <span key={w} className="by keyword">{w}</span>)}</p>
              )}
            </>
          )}
          <blockquote>{highlight(result.snippet, result.matched_words)}</blockquote>
        </div>

        {error && <p className="error">{error}</p>}
        {!record && !error && <p className="muted">Loading the full record…</p>}
        {record && (
          <dl className="record">
            {Object.entries(record.fields).map(([k, v]) => (
              <div key={k} className={typeof v === "string" && v.length > 80 ? "wide" : ""}>
                <dt>{label(k)}</dt>
                <dd>{v === null ? "—" : formatValue(k, v)}</dd>
              </div>
            ))}
          </dl>
        )}
      </aside>
    </div>
  );
}
