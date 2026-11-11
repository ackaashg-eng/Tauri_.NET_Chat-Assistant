/** Small display helpers shared by the chat, tools and admin pages. */

/** "duration_hours" / "durationHours" -> "Duration hours"; "erp_module" -> "ERP module". */
export function label(key: string) {
  const s = key.replace(/_/g, " ").replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase()
    .replace(/\berp\b/g, "ERP").replace(/\bid\b/g, "ID").replace(/\bmcp\b/g, "MCP");
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export function formatValue(key: string, v: unknown): string {
  if (typeof v === "boolean") return v ? "Yes" : "No";
  if (typeof v === "number" && key.includes("price")) return v.toLocaleString(undefined, { style: "currency", currency: "USD" });
  if (key === "warranty_months" && typeof v === "number") return `${v} months`;
  if (key === "tags" && typeof v === "string") return v.split(",").join(", ");
  return String(v);
}

/** Wraps the matched words in <mark> so people can see why a row matched. */
export function highlight(text: string, words: string[]) {
  const clean = words.filter((w) => w.length > 1).map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  if (!clean.length) return text;
  const rx = new RegExp(`(${clean.join("|")})`, "gi");
  return text.split(rx).map((part, i) => (i % 2 === 1 ? <mark key={i}>{part}</mark> : part));
}

/** Match score drawn as a meter, with the minimum-match threshold marked on it. */
export function ScoreGauge({ score, threshold }: { score: number; threshold: number }) {
  const pct = Math.max(0, Math.min(1, score)) * 100;
  const strength = score >= 0.7 ? "Strong" : score >= 0.45 ? "Good" : "Weak";
  return (
    <div className="gauge" aria-label={`Match score ${score.toFixed(2)}, ${strength.toLowerCase()}`}>
      <div className="gauge-top">
        <span className="gauge-value">{score.toFixed(2)}</span>
        <span className="gauge-word">{strength}</span>
      </div>
      <div className="gauge-track">
        <div className="gauge-fill" style={{ width: `${pct}%` }} />
        <div className="gauge-threshold" style={{ left: `${threshold * 100}%` }} title={`Minimum ${threshold.toFixed(2)}`} />
      </div>
    </div>
  );
}

export function cell(v: unknown): string {
  if (v === null || v === undefined || v === "") return "—";
  if (typeof v === "boolean") return v ? "Yes" : "No";
  if (typeof v === "number") return Number.isInteger(v) ? v.toLocaleString() : v.toLocaleString(undefined, { maximumFractionDigits: 2 });
  if (Array.isArray(v)) return v.map(cell).join(", ");
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}

const isPlainObject = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);

/** Renders any JSON from a tool: lists of objects become tables, objects become label/value pairs. */
export function Value({ value, onRowClick }: { value: unknown; onRowClick?: (row: Record<string, unknown>) => void }) {
  if (value === null || value === undefined) return <span className="muted">—</span>;
  if (Array.isArray(value)) {
    if (value.length === 0) return <p className="muted">Nothing found.</p>;
    if (value.every(isPlainObject)) return <DataTable rows={value as Record<string, unknown>[]} onRowClick={onRowClick} />;
    return <span>{value.map(cell).join(", ")}</span>;
  }
  if (isPlainObject(value)) {
    const entries = Object.entries(value);
    const simple = entries.filter(([, v]) => !Array.isArray(v) && !isPlainObject(v));
    const nested = entries.filter(([, v]) => Array.isArray(v) || isPlainObject(v));
    return (
      <div className="obj">
        {simple.length > 0 && (
          <dl className="kv result-kv">
            {simple.map(([k, v]) => (
              <div key={k} className={typeof v === "string" && v.length > 60 ? "wide" : ""}><dt>{label(k)}</dt><dd>{cell(v)}</dd></div>
            ))}
          </dl>
        )}
        {nested.map(([k, v]) => (
          <div key={k} className="nested">
            <h4>{label(k)}{Array.isArray(v) ? ` (${v.length})` : ""}</h4>
            <Value value={v} onRowClick={onRowClick} />
          </div>
        ))}
      </div>
    );
  }
  return <span>{cell(value)}</span>;
}

export function DataTable({ rows, onRowClick }: { rows: Record<string, unknown>[]; onRowClick?: (row: Record<string, unknown>) => void }) {
  const columns = Array.from(new Set(rows.flatMap((r) => Object.keys(r))));
  return (
    <div className="table-wrap">
      <table className="data">
        <thead><tr>{columns.map((c) => <th key={c}>{label(c)}</th>)}</tr></thead>
        <tbody>
          {rows.map((r, i) => (
            <tr key={i} className={onRowClick ? "clickable" : ""} onClick={onRowClick ? () => onRowClick(r) : undefined}
                tabIndex={onRowClick ? 0 : undefined}
                onKeyDown={onRowClick ? (e) => { if (e.key === "Enter") onRowClick(r); } : undefined}>
              {columns.map((c) => {
                const v = r[c];
                const text = cell(v);
                return (
                  <td key={c} className={typeof v === "number" ? "num" : text.length > 70 ? "long" : ""} title={text.length > 70 ? text : undefined}>
                    {text}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
