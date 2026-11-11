import { useEffect, useMemo, useState, type FormEvent } from "react";
import { api, type JsonSchema, type ToolInfo } from "../api";
import { label, Value } from "./format";

/**
 * Runs any MCP tool the C# server offers. The form is built from the tool's JSON input
 * schema (from tools/list), so new tools added to the server appear here automatically.
 */

type Group = "Search" | "Look up" | "Reports" | "Change data";
const GROUPS: Group[] = ["Search", "Look up", "Reports", "Change data"];
const LONG_TEXT = new Set(["description", "comments", "notes", "agenda", "outcome", "actionItems", "query"]);
const HIDDEN = new Set(["performedBy"]); // filled in by the server with the signed-in user

function groupOf(t: ToolInfo): Group {
  if (t.read_only === false) return "Change data";
  if (t.name.includes("search") || t.name === "explain_query") return "Search";
  if (t.name.endsWith("_report") || t.name === "consultant_workload") return "Reports";
  return "Look up";
}

type Options = Record<string, unknown>;

/** Dropdown choices for well-known ERP fields, taken from the erp_options tool. */
function choicesFor(tool: string, field: string, o: Options | null): string[] | null {
  if (!o) return null;
  const list = (k: string) => (Array.isArray(o[k]) ? (o[k] as string[]) : null);
  switch (field) {
    case "module": return list("modules");
    case "type": return list("consultation_types");
    case "priority": return list("priorities");
    case "status": return tool.includes("order") ? list("order_statuses") : list("consultation_statuses");
    case "mode": return tool === "add_meeting" ? list("meeting_modes") : ["semantic", "keyword", "pattern"];
    case "area": return ["all", "consultations", "meetings", "feedback", "orders"];
    case "groupBy":
      if (tool === "revenue_report") return list("revenue_report_groups");
      if (tool === "feedback_report") return list("feedback_report_groups");
      if (tool === "consultation_report") return list("consultation_report_groups");
      return null;
    default: return null;
  }
}

function baseType(s: JsonSchema): string {
  const types = Array.isArray(s.type) ? s.type : [s.type ?? "string"];
  return types.find((t) => t !== "null") ?? "string";
}

export default function ToolsPage({ ready }: { ready: boolean }) {
  const [tools, setTools] = useState<ToolInfo[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [options, setOptions] = useState<Options | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!ready) return;
    api.tools().then((t) => {
      setTools(t);
      setSelected((cur) => cur ?? t.find((x) => x.name === "list_consultations")?.name ?? t[0]?.name ?? null);
      if (t.some((x) => x.name === "erp_options"))
        api.runTool("erp_options", {}).then((o) => setOptions(o as Options)).catch(() => {});
    }).catch((e) => setError(e.message));
  }, [ready]);

  const tool = tools.find((t) => t.name === selected) ?? null;
  const grouped = useMemo(() => GROUPS.map((g) => ({ g, items: tools.filter((t) => groupOf(t) === g) })), [tools]);

  if (!ready) return <p className="notice">The search server is starting…</p>;

  return (
    <div className="tools-page">
      <nav className="tool-list" aria-label="Tools">
        {error && <p className="error">{error}</p>}
        {grouped.map(({ g, items }) => items.length > 0 && (
          <div key={g}>
            <h3>{g}</h3>
            <ul>
              {items.map((t) => (
                <li key={t.name}>
                  <button className={t.name === selected ? "on" : ""} onClick={() => setSelected(t.name)}>
                    {t.title ?? label(t.name)}
                    {t.allowed === false && <span className="lock">admin</span>}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>
      <section className="tool-panel">
        {tool ? <ToolRunner key={tool.name} tool={tool} options={options} /> : <p className="muted">Choose a tool on the left.</p>}
      </section>
    </div>
  );
}

function ToolRunner({ tool, options }: { tool: ToolInfo; options: Options | null }) {
  const schema = tool.input_schema ?? {};
  const fields = Object.entries(schema.properties ?? {}).filter(([n]) => !HIDDEN.has(n));
  const required = new Set(schema.required ?? []);
  const changes = tool.read_only === false;

  const [values, setValues] = useState<Record<string, string | boolean>>(() =>
    Object.fromEntries(fields.map(([n, s]) => [n, baseType(s) === "boolean" ? s.default === true : s.default == null ? "" : String(s.default)])));
  const [result, setResult] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function run(e: FormEvent) {
    e.preventDefault();
    const args: Record<string, unknown> = {};
    for (const [name, s] of fields) {
      const v = values[name];
      const t = baseType(s);
      if (t === "boolean") { args[name] = v === true; continue; }
      const text = String(v ?? "").trim();
      if (!text) continue;
      if (t === "integer") args[name] = parseInt(text, 10);
      else if (t === "number") args[name] = parseFloat(text);
      else args[name] = text;
    }
    setBusy(true); setError(null); setResult(null);
    try { setResult(await api.runTool(tool.name, args)); }
    catch (err) { setError((err as Error).message); }
    finally { setBusy(false); }
  }

  return (
    <>
      <header className="tool-head">
        <h2>{tool.title ?? label(tool.name)}</h2>
        <code>{tool.name}</code>
      </header>
      {tool.description && <p className="tool-desc">{tool.description}</p>}
      {changes && <p className="hint">This saves a change to the database. It is recorded under your name.</p>}

      <form className="tool-form" onSubmit={run}>
        {fields.length === 0 && <p className="muted">This tool has no settings.</p>}
        {fields.map(([name, s]) => {
          const t = baseType(s);
          const choices = choicesFor(tool.name, name, options);
          const id = `f-${name}`;
          const isRequired = required.has(name);
          const common = { id, title: s.description, required: isRequired && t !== "boolean" };
          return (
            <div key={name} className={`field ${LONG_TEXT.has(name) ? "wide" : ""} ${t === "boolean" ? "check" : ""}`}>
              {t === "boolean" ? (
                <label htmlFor={id}>
                  <input type="checkbox" id={id} checked={values[name] === true}
                         onChange={(e) => setValues({ ...values, [name]: e.target.checked })} />
                  {label(name)}
                </label>
              ) : (
                <>
                  <label htmlFor={id}>{label(name)}{isRequired && <span className="req" aria-label="required"> *</span>}</label>
                  {choices ? (
                    <select {...common} value={String(values[name] ?? "")} onChange={(e) => setValues({ ...values, [name]: e.target.value })}>
                      {!isRequired && <option value="">Any</option>}
                      {choices.map((c) => <option key={c} value={c}>{c}</option>)}
                    </select>
                  ) : LONG_TEXT.has(name) ? (
                    <textarea {...common} rows={name === "query" ? 2 : 3} value={String(values[name] ?? "")}
                              onChange={(e) => setValues({ ...values, [name]: e.target.value })} />
                  ) : (
                    <input {...common} type={t === "integer" || t === "number" ? "number" : "text"} step={t === "number" ? "any" : undefined}
                           placeholder={/^(from|to|scheduledStart|meetingAt)$/.test(name) ? "YYYY-MM-DD" : undefined}
                           value={String(values[name] ?? "")} onChange={(e) => setValues({ ...values, [name]: e.target.value })} />
                  )}
                  {s.description && <small>{s.description}</small>}
                </>
              )}
            </div>
          );
        })}
        <div className="field wide">
          <button className="primary" disabled={busy || tool.allowed === false}>
            {busy ? "Working…" : tool.title ?? "Run"}
          </button>
          {tool.allowed === false && <span className="hint"> Only administrators can run this tool.</span>}
        </div>
      </form>

      {error && <p className="error banner" role="alert">{error}</p>}
      {result !== null && <ResultView result={result} />}
    </>
  );
}

// ------------------------------------------------------------------------------- results

function ResultView({ result }: { result: unknown }) {
  const message = typeof result === "object" && result && "message" in result ? String((result as any).message) : null;
  return (
    <section className="tool-result" aria-label="Result">
      {message && <p className="ok banner" role="status">{message}</p>}
      <Value value={message ? omit(result as Record<string, unknown>, "message") : result} />
      <details className="raw">
        <summary>Show the raw JSON the MCP server returned</summary>
        <pre>{JSON.stringify(result, null, 2)}</pre>
      </details>
    </section>
  );
}

function omit(o: Record<string, unknown>, key: string) {
  const { [key]: _, ...rest } = o;
  return rest;
}
