// Small typed client for the Rust backend (spil-backend). The token lives in sessionStorage,
// so closing the browser tab signs the user out.

export type Role = "admin" | "user";
export interface User { id: number; username: string; role: Role; active: boolean; created_at: string }

export interface SearchResult {
  source: string;
  id: number;
  title: string;
  score: number;
  matched_by: string[];          // "concepts", "words" or "pattern"
  matched_concepts: string[];
  matched_words: string[];
  snippet: string;
  fields: Record<string, unknown>;
}
export type Mode = "semantic" | "keyword" | "pattern";
export interface SearchResponse {
  query: string; mode: Mode; method: string; top_k: number; min_score: number;
  source: string | null; query_concepts: string[]; query_words: string[];
  candidates: number; took_ms: number; results: SearchResult[];
}
export interface Source { name: string; display_name: string; description: string; table: string; records: number; display_columns: string[] }
export interface IndexStatus {
  method: string; records: number; concepts: number; vocabulary: number; rules_file: string;
  database: string; last_indexed: string | null; recommended_min_score: number;
}
export interface Explanation { query: string; words: { word: string; compared_as: string }[]; concepts: { concept: string; matched_pattern: string }[] }
export interface Status { connected: boolean; transport: string; index: IndexStatus | null; error: string | null }
export interface RecordDetail { source: string; display_name: string; id: number; title: string; fields: Record<string, unknown> }
export interface JsonSchema {
  type?: string | string[]; description?: string; default?: unknown; enum?: unknown[];
  properties?: Record<string, JsonSchema>; required?: string[]; items?: JsonSchema;
}
export interface ToolInfo { name: string; title: string | null; description: string | null; input_schema: JsonSchema; read_only: boolean | null; allowed?: boolean }
export interface McpInfo { server: Record<string, any>; tools: ToolInfo[]; command: string; args: string[]; transport: string }
export interface ChatResponse {
  reply: string;
  tool: string;
  arguments: Record<string, unknown>;
  kind: "search" | "rows" | "record" | "report";
  rows_key: string | null;
  data: any;
  took_ms: number;
}
export interface HistoryItem { query: string; mode: string; results: number; created_at: string }

const TOKEN_KEY = "spil.token";

export class ApiError extends Error {
  constructor(public status: number, message: string) { super(message); }
}

function getToken(): string | null {
  try { return sessionStorage.getItem(TOKEN_KEY); } catch { return null; }
}
export function setToken(token: string | null) {
  try { token ? sessionStorage.setItem(TOKEN_KEY, token) : sessionStorage.removeItem(TOKEN_KEY); } catch { /* storage blocked */ }
}
export const hasToken = () => !!getToken();

let onUnauthorized: () => void = () => {};
export const setUnauthorizedHandler = (fn: () => void) => { onUnauthorized = fn; };

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = {};
  const token = getToken();
  if (token) headers.Authorization = `Bearer ${token}`;
  if (body !== undefined) headers["Content-Type"] = "application/json";

  let res: Response;
  try {
    res = await fetch(`/api${path}`, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
  } catch {
    throw new ApiError(0, "Can't reach the server. Check your network connection.");
  }
  if (res.status === 204) return undefined as T;
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    if (res.status === 401 && path !== "/auth/login") { setToken(null); onUnauthorized(); }
    throw new ApiError(res.status, readDetail(data) ?? `Request failed (${res.status})`);
  }
  return data as T;
}

function readDetail(data: any): string | null {
  if (typeof data?.detail === "string") return data.detail;
  if (Array.isArray(data?.detail) && data.detail[0]?.msg) {
    const d = data.detail[0];
    const field = Array.isArray(d.loc) ? d.loc[d.loc.length - 1] : "";
    return `${field ? field + ": " : ""}${String(d.msg).replace(/^Value error, /, "")}`;
  }
  return null;
}

export const api = {
  login: (username: string, password: string) =>
    request<{ access_token: string; user: User }>("POST", "/auth/login", { username, password }),
  me: () => request<User>("GET", "/auth/me"),
  changePassword: (current_password: string, new_password: string) =>
    request<void>("POST", "/auth/change-password", { current_password, new_password }),

  search: (q: { query: string; top_k: number; min_score: number | null; source: string | null; mode: string }) =>
    request<SearchResponse>("POST", "/search", q),
  history: () => request<HistoryItem[]>("GET", "/search/history"),
  chat: (q: { message: string; top_k?: number; min_score?: number | null; source?: string | null; mode?: Mode }) =>
    request<ChatResponse>("POST", "/chat", q),
  sources: () => request<{ sources: Source[] }>("GET", "/sources"),
  record: (source: string, id: number) => request<RecordDetail>("GET", `/records/${encodeURIComponent(source)}/${id}`),
  status: () => request<Status>("GET", "/status"),

  tools: () => request<ToolInfo[]>("GET", "/tools"),
  runTool: (name: string, args: Record<string, unknown>) => request<unknown>("POST", `/tools/${encodeURIComponent(name)}`, { arguments: args }),
  mcpInfo: () => request<McpInfo>("GET", "/admin/mcp"),
  reindex: () => request<Record<string, unknown>>("POST", "/admin/reindex"),
  explain: (q: string) => request<Explanation>("GET", `/search/explain?q=${encodeURIComponent(q)}`),
  users: () => request<User[]>("GET", "/admin/users"),
  createUser: (username: string, password: string, role: Role) => request<User>("POST", "/admin/users", { username, password, role }),
  updateUser: (id: number, patch: { role?: Role; active?: boolean }) => request<User>("PATCH", `/admin/users/${id}`, patch),
  resetPassword: (id: number, password: string) => request<void>("POST", `/admin/users/${id}/password`, { password }),
  deleteUser: (id: number) => request<void>("DELETE", `/admin/users/${id}`),
};
