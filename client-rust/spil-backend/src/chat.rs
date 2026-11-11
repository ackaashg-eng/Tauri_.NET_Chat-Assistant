//! Chat without an AI model: regular expressions decide which MCP tool answers a message.
//!
//! ```text
//! "show in progress consultations for Coral Coast Retail" -> list_consultations(status, client)
//! "revenue by month"                                      -> revenue_report(groupBy = month)
//! "/E-\d{3}/"                                             -> semantic_search(mode = pattern)
//! anything else                                           -> semantic_search (meaning search)
//! ```
//!
//! Every reply says which tool was used and with which arguments. The chat only READS data;
//! changes are made on the Tools page. (Port of the old backend/app/chat.py.)

use std::sync::LazyLock;

use regex::Regex;
use serde_json::{json, Map, Value};

fn re(p: &str) -> Regex {
    Regex::new(p).expect("valid chat regex")
}

/// Statuses a consultation can be asked for by name (checked in this order).
const CONSULTATION_STATUS: &[(&str, &str)] = &[
    ("requested", "Requested"), ("new", "Requested"), ("pending", "Requested"),
    ("scheduled", "Scheduled"), ("booked", "Scheduled"), ("planned", "Scheduled"),
    ("in progress", "In progress"), ("in-progress", "In progress"), ("ongoing", "In progress"), ("active", "In progress"),
    ("completed", "Completed"), ("finished", "Completed"), ("done", "Completed"), ("closed", "Completed"),
    ("cancelled", "Cancelled"), ("canceled", "Cancelled"),
];
const OPEN_STATUSES: &[&str] = &["Requested", "Scheduled", "In progress"];

const ORDER_STATUS: &[(&str, &[&str])] = &[
    ("unpaid", &["Confirmed", "Invoiced"]), ("outstanding", &["Confirmed", "Invoiced"]), ("owed", &["Confirmed", "Invoiced"]),
    ("quoted", &["Quoted"]), ("quotes", &["Quoted"]), ("quotations", &["Quoted"]), ("confirmed", &["Confirmed"]),
    ("invoiced", &["Invoiced"]), ("paid", &["Paid"]), ("cancelled", &["Cancelled"]), ("canceled", &["Cancelled"]),
];

static MODULES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (re(r"\b(hr|payroll|salar\w*|wages?)\b"), "HR & Payroll"),
        (re(r"\b(finance|accounting|accounts|ledger)\b"), "Finance"),
        (re(r"\b(inventory|stock|warehouse)\b"), "Inventory"),
        (re(r"\b(manufacturing|production)\b"), "Manufacturing"),
        (re(r"\b(purchasing|procurement)\b"), "Purchasing"),
        (re(r"\b(integration|api)\b"), "Integration"),
        (re(r"\b(reporting|dashboards?)\b"), "Reporting"),
        (re(r"\bcrm\b"), "CRM"),
        (re(r"\bprojects?\b"), "Projects"),
    ]
});

const LIST_VERB: &str = r"\b(list|show|display|give me|get|find|which|what|all|any|see|view)\b";

struct Rules {
    pattern: Regex,
    consultation_id: Regex,
    workload: Regex,
    revenue: Regex,
    revenue2: Regex,
    satisfaction: Regex,
    consultation_count: Regex,
    meetings: Regex,
    upcoming: Regex,
    list_verb: Regex,
    list_or_who: Regex,
    feedback: Regex,
    search_about: Regex,
    follow_up: Regex,
    bad: Regex,
    good: Regex,
    orders: Regex,
    consultants: Regex,
    clients: Regex,
    consultation_word: Regex,
    using: Regex,
    open: Regex,
    urgent: Regex,
    consultations: Regex,
    client_name: Regex,
    group: Regex,
}

static R: LazyLock<Rules> = LazyLock::new(|| Rules {
    pattern: re(r"^/(.+)/$"),
    consultation_id: re(r"\b(?:consultation|booking|case|job)\s*(?:#|no\.?|number|id)?\s*(\d{1,7})\b"),
    workload: re(r"\b(workload|how busy|busiest|capacity|utili[sz]ation|who has (the )?most work)\b"),
    revenue: re(r"\b(revenue|income|earnings|turnover|billing|how much (have|did) we (earn|bill|make|invoice))\b"),
    revenue2: re(r"\b(order|sales)s? (report|summary|totals?)\b"),
    satisfaction: re(r"\b(satisfaction|average rating|ratings? (report|summary|by|per)|feedback (report|summary|by|per)|nps)\b"),
    consultation_count: re(r"\b(how many consultations|number of consultations|consultations? (report|summary|breakdown|count|by|per))\b"),
    meetings: re(r"\bmeetings?\b"),
    upcoming: re(r"\b(upcoming|next|coming|future|planned|scheduled|diary|calendar|this week|later)\b"),
    list_verb: re(LIST_VERB),
    list_or_who: re(&format!(r"{LIST_VERB}|\bwho\b")),
    feedback: re(r"\b(feedback|ratings?|reviews?|comments)\b"),
    search_about: re(r"\b(search|about)\b"),
    follow_up: re(r"\bfollow[- ]?ups?\b"),
    bad: re(r"\b(bad|poor|low|negative|worst|unhappy|angry|complaints?)\b"),
    good: re(r"\b(good|great|best|positive|happy|top)\b"),
    orders: re(r"\b(orders?|quotes?|quotations?|invoices?)\b"),
    consultants: re(r"\bconsultants\b"),
    clients: re(r"\bclients\b"),
    consultation_word: re(r"\bconsultations?\b"),
    using: re(r"(?i)\b(?:using|on|in|with)\s+([\w &.-]+)$"),
    open: re(r"\b(open|unfinished|outstanding|current)\b"),
    urgent: re(r"\b(urgent|critical|high[- ]priority)\b"),
    consultations: re(r"\b(consultations?|bookings?|jobs)\b"),
    client_name: re(r"\b(?:for|from|of|with|at|by)\s+((?:[A-Z][\w&'().-]*)(?:\s+(?:[A-Z][\w&'().-]*|&))*)"),
    group: re(r"(?i)\b(?:by|per|for each|grouped by|split by)\s+(\w+)"),
});

fn word(w: &str) -> Regex {
    re(&format!(r"\b{}\b", regex::escape(w)))
}

/// What to run and how to describe it.
#[derive(Debug, Clone)]
pub struct Plan {
    pub tool: String,
    pub arguments: Map<String, Value>,
    pub kind: &'static str, // search | rows | record | report
    pub title: String,
    pub rows_key: Option<&'static str>,
    /// Filter rows after the call, e.g. only open statuses.
    pub keep: Option<(&'static str, Vec<&'static str>)>,
}

impl Plan {
    fn new(tool: &str, arguments: Value, kind: &'static str, title: impl Into<String>, rows_key: Option<&'static str>) -> Self {
        let arguments = match arguments {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        Plan { tool: tool.into(), arguments, kind, title: title.into(), rows_key, keep: None }
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub top_k: i64,
    pub min_score: Option<f64>,
    pub source: Option<String>,
    pub mode: String,
}

impl Default for Options {
    fn default() -> Self {
        Self { top_k: 8, min_score: None, source: None, mode: "semantic".into() }
    }
}

/// A capitalised name after for/from/of/with: "for Coral Coast Retail" -> "Coral Coast Retail".
fn client(text: &str) -> Option<String> {
    let name = R.client_name.captures(text)?.get(1)?.as_str().trim().to_string();
    let skip = ["i", "me", "us", "status", "month", "client", "consultant"];
    (!skip.contains(&name.to_lowercase().as_str())).then_some(name)
}

fn group(text: &str, allowed: &[&str], default: &str) -> String {
    if let Some(c) = R.group.captures(text) {
        let w = c[1].to_lowercase();
        let w = match w.as_str() {
            "customer" | "clients" | "customers" => "client".to_string(),
            "months" | "monthly" => "month".to_string(),
            "consultants" => "consultant".to_string(),
            "modules" => "module".to_string(),
            "services" => "service".to_string(),
            "types" => "type".to_string(),
            _ => w,
        };
        if allowed.contains(&w.as_str()) {
            return w;
        }
    }
    default.to_string()
}

fn status(low: &str) -> Option<&'static str> {
    CONSULTATION_STATUS.iter().find(|(w, _)| word(w).is_match(low)).map(|(_, s)| *s)
}

pub fn search_plan(query: &str, o: &Options) -> Plan {
    let mut args = json!({ "query": query, "topK": o.top_k, "mode": o.mode });
    if let (Some(m), true) = (o.min_score, o.mode != "pattern") {
        args["minScore"] = json!(m);
    }
    if let Some(s) = &o.source {
        args["source"] = json!(s);
    }
    let title = if o.mode == "pattern" { "matched the pattern" } else { "matched" };
    Plan::new("semantic_search", args, "search", title, None)
}

pub fn route(message: &str, o: &Options) -> Plan {
    let text = message.trim();
    let low = text.to_lowercase();
    let low = low.as_str();
    let search = |q: &str, mode: &str| search_plan(q, &Options { mode: mode.into(), ..o.clone() });

    // /regex/ -> pattern search
    if let Some(c) = R.pattern.captures(text) {
        return search(&c[1], "pattern");
    }
    if o.mode == "pattern" {
        return search(text, "pattern");
    }

    // "consultation 12", "booking #4"
    if let Some(c) = R.consultation_id.captures(low) {
        let id: i64 = c[1].parse().unwrap_or(0);
        return Plan::new("get_consultation", json!({ "id": id }), "record", format!("Consultation {id}"), None);
    }

    if R.workload.is_match(low) {
        return Plan::new("consultant_workload", json!({}), "report", "Consultant workload", None);
    }

    if R.revenue.is_match(low) || R.revenue2.is_match(low) {
        let g = group(text, &["client", "month", "service", "status", "module", "consultant"], "client");
        return Plan::new("revenue_report", json!({ "groupBy": g }), "report", format!("Revenue by {g}"), None);
    }

    if R.satisfaction.is_match(low) {
        let g = group(text, &["consultant", "client", "module", "type", "month"], "consultant");
        return Plan::new("feedback_report", json!({ "groupBy": g }), "report", format!("Client satisfaction by {g}"), None);
    }

    if R.consultation_count.is_match(low) {
        let g = group(text, &["status", "module", "type", "priority", "consultant", "client", "month"], "status");
        return Plan::new("consultation_report", json!({ "groupBy": g }), "report", format!("Consultations by {g}"), None);
    }

    // meetings
    if R.meetings.is_match(low) {
        let mut args = json!({ "limit": 50 });
        let c = client(text);
        if let Some(c) = &c {
            args["client"] = json!(c);
        }
        if R.upcoming.is_match(low) {
            args["upcomingOnly"] = json!(true);
            return Plan::new("list_meetings", args, "rows", "Upcoming meetings", Some("meetings"));
        }
        if R.list_verb.is_match(low) || c.is_some() {
            return Plan::new("list_meetings", args, "rows", "Meetings", Some("meetings"));
        }
    }

    // feedback lists
    if R.feedback.is_match(low) && !R.search_about.is_match(low) {
        let mut args = json!({ "limit": 50 });
        if let Some(c) = client(text) {
            args["client"] = json!(c);
        }
        if R.follow_up.is_match(low) {
            args["followUpOnly"] = json!(true);
            return Plan::new("list_feedback", args, "rows", "Feedback that needs a follow-up", Some("feedback"));
        }
        if R.bad.is_match(low) {
            args["maxRating"] = json!(2);
            return Plan::new("list_feedback", args, "rows", "Low-rated feedback (1–2 stars)", Some("feedback"));
        }
        if R.good.is_match(low) {
            args["minRating"] = json!(4);
            return Plan::new("list_feedback", args, "rows", "Positive feedback (4–5 stars)", Some("feedback"));
        }
        if R.list_verb.is_match(low) {
            return Plan::new("list_feedback", args, "rows", "Feedback", Some("feedback"));
        }
    }

    // orders / quotes / invoices
    let order_status = ORDER_STATUS.iter().find(|(w, _)| word(w).is_match(low)).map(|(_, s)| *s);
    if R.orders.is_match(low) && (R.list_verb.is_match(low) || order_status.is_some()) {
        let mut args = json!({ "limit": 100 });
        if let Some(c) = client(text) {
            args["client"] = json!(c);
        }
        let title = match order_status {
            Some([one]) => {
                args["status"] = json!(one);
                format!("{one} orders")
            }
            Some(_) => "Unpaid orders".to_string(),
            None => "All orders".to_string(),
        };
        let mut plan = Plan::new("list_orders", args, "rows", title, Some("orders"));
        if let Some(statuses) = order_status.filter(|s| s.len() > 1) {
            plan.keep = Some(("status", statuses.to_vec()));
        }
        return plan;
    }

    // people
    if R.consultants.is_match(low) && R.list_or_who.is_match(low) {
        return Plan::new("list_consultants", json!({ "activeOnly": true }), "rows", "Active consultants", Some("consultants"));
    }
    if R.clients.is_match(low) && R.list_or_who.is_match(low) && !R.consultation_word.is_match(low) {
        let mut args = json!({ "limit": 100 });
        let mut title = "Clients".to_string();
        if let Some(c) = R.using.captures(text) {
            let s = c[1].trim().to_string();
            title = format!("Clients matching “{s}”");
            args["search"] = json!(s);
        }
        return Plan::new("list_clients", args, "rows", title, Some("clients"));
    }

    // consultations with filters
    let st = status(low);
    let wants_open = R.open.is_match(low);
    let urgent = R.urgent.is_match(low);
    if R.consultations.is_match(low) && (R.list_verb.is_match(low) || st.is_some() || wants_open || urgent) {
        let mut args = json!({ "limit": 100 });
        if let Some(s) = st {
            args["status"] = json!(s);
        }
        if urgent {
            args["priority"] = json!("Urgent");
        }
        let c = client(text);
        if let Some(c) = &c {
            args["client"] = json!(c);
        }
        let module = MODULES.iter().find(|(r, _)| r.is_match(low)).map(|(_, m)| *m);
        if let Some(m) = module {
            args["module"] = json!(m);
        }
        let words: Vec<&str> = [st.or(wants_open.then_some("Open")), urgent.then_some("urgent"), module]
            .into_iter()
            .flatten()
            .collect();
        let mut title = if words.is_empty() { "Consultations".to_string() } else { format!("{} consultations", words.join(" ")) };
        if let Some(first) = title.get(..1) {
            title = first.to_uppercase() + &title[1..];
        }
        if let Some(c) = c {
            title += &format!(" for {c}");
        }
        let mut plan = Plan::new("list_consultations", args, "rows", title, Some("consultations"));
        if wants_open && st.is_none() {
            plan.keep = Some(("status", OPEN_STATUSES.to_vec()));
        }
        return plan;
    }

    search(text, &o.mode)
}

/// The assistant's sentence, plus the (possibly filtered) data to show.
pub fn summarise(plan: &Plan, result: Value) -> (String, Value) {
    match plan.kind {
        "search" => {
            let n = result.get("results").and_then(Value::as_array).map_or(0, Vec::len);
            if n == 0 {
                let q = result.get("query").and_then(Value::as_str).unwrap_or("");
                return (
                    format!("I couldn't find anything that {} “{q}”. Try fewer words, another section, or a lower minimum match.", plan.title),
                    result,
                );
            }
            let concepts: Vec<&str> = result
                .get("query_concepts")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let about = if concepts.is_empty() { String::new() } else { format!(" I read this as {}.", concepts.join(", ")) };
            let noun = if n == 1 { "result" } else { "results" };
            (format!("I found {n} {noun} that {} your question.{about}", plan.title), result.clone())
        }
        "rows" => {
            let key = plan.rows_key.unwrap_or("");
            let mut rows: Vec<Value> = result.get(key).and_then(Value::as_array).cloned().unwrap_or_default();
            if let Some((field, allowed)) = &plan.keep {
                rows.retain(|r| r.get(*field).and_then(Value::as_str).is_some_and(|v| allowed.contains(&v)));
            }
            let n = rows.len();
            let data = json!({ "count": n, key: rows });
            if n == 0 {
                (format!("{}: none found.", plan.title), data)
            } else {
                (format!("{}: {n} found.", plan.title), data)
            }
        }
        "record" => {
            let id = result.get("id").cloned().unwrap_or(Value::Null);
            let title = result.get("title").and_then(Value::as_str).unwrap_or("");
            (format!("Here is everything about consultation {id}: {title}."), result)
        }
        _ => {
            let n = result.get("rows").and_then(Value::as_array).map_or(0, Vec::len);
            let noun = if n == 1 { "row" } else { "rows" };
            (format!("{} — {n} {noun}.", plan.title), result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(msg: &str) -> Plan {
        route(msg, &Options::default())
    }

    #[test]
    fn default_is_meaning_search() {
        let p = r("my parcel never showed up");
        assert_eq!(p.tool, "semantic_search");
        assert_eq!(p.arguments["query"], "my parcel never showed up");
    }

    #[test]
    fn regex_between_slashes_is_pattern_search() {
        let p = r(r"/E-\d{3}/");
        assert_eq!(Value::Object(p.arguments), json!({ "query": r"E-\d{3}", "topK": 8, "mode": "pattern" }));
    }

    #[test]
    fn consultation_lists_with_filters() {
        let p = r("show in progress consultations for Coral Coast Retail");
        assert_eq!(p.tool, "list_consultations");
        assert_eq!(p.arguments["status"], "In progress");
        assert_eq!(p.arguments["client"], "Coral Coast Retail");
        assert_eq!(p.title, "In progress consultations for Coral Coast Retail");
        let p = r("which urgent payroll consultations are open?");
        assert_eq!(p.arguments["priority"], "Urgent");
        assert_eq!(p.arguments["module"], "HR & Payroll");
        assert!(p.keep.is_some());
    }

    #[test]
    fn reports() {
        assert_eq!(Value::Object(r("revenue by month").arguments), json!({ "groupBy": "month" }));
        assert_eq!(Value::Object(r("show the revenue report").arguments), json!({ "groupBy": "client" }));
        assert_eq!(r("customer satisfaction by module").tool, "feedback_report");
        assert_eq!(Value::Object(r("how many consultations per consultant").arguments), json!({ "groupBy": "consultant" }));
        assert_eq!(r("who is the busiest consultant?").tool, "consultant_workload");
    }

    #[test]
    fn other_lists() {
        assert_eq!(r("upcoming meetings").arguments["upcomingOnly"], true);
        assert_eq!(r("show bad feedback").arguments["maxRating"], 2);
        let p = r("list unpaid orders");
        assert_eq!(p.tool, "list_orders");
        assert_eq!(p.keep, Some(("status", vec!["Confirmed", "Invoiced"])));
        assert_eq!(p.title, "Unpaid orders");
        assert_eq!(r("show paid orders").arguments["status"], "Paid");
        assert_eq!(Value::Object(r("show consultation 12").arguments), json!({ "id": 12 }));
        assert_eq!(r("list clients using Odoo").arguments["search"], "Odoo");
        assert_eq!(r("list consultants").tool, "list_consultants");
        assert_eq!(r("orders").tool, "semantic_search");
    }

    #[test]
    fn summary_filters_rows() {
        let p = r("list unpaid orders");
        let (text, data) = summarise(&p, json!({ "orders": [{ "status": "Paid" }, { "status": "Invoiced" }, { "status": "Confirmed" }] }));
        assert_eq!(data["count"], 2);
        assert!(text.starts_with("Unpaid orders: 2"));
    }
}
