using System.ComponentModel;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using ModelContextProtocol;
using ModelContextProtocol.Server;
using SemanticSearch.McpServer.Data;
using SemanticSearch.McpServer.Search;

namespace SemanticSearch.McpServer.Tools;

/// <summary>
/// MCP tools for the ERP consultancy data: search, filter/list, reports, and create/update.
/// Every tool returns JSON text. Input mistakes come back as clear MCP tool errors.
/// Dates are "YYYY-MM-DD" or "YYYY-MM-DD HH:mm"; a "to" date includes that whole day.
/// </summary>
[McpServerToolType]
public sealed class ErpTools(ErpRepository erp, SearchEngine engine, Indexer indexer, ILogger<ErpTools> logger)
{
    private const string DateHelp = "Format YYYY-MM-DD (or YYYY-MM-DD HH:mm).";
    private const string ByHelp = "Name of the person making the change, saved for the audit trail.";
    private static readonly JsonSerializerOptions Json = SearchTools.Json;

    // ================================================================================== search

    [McpServerTool(Name = "search_consultations", Title = "Search consultations", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Searches ERP consultations, meeting notes, client feedback and orders by meaning (regex rules), " +
                 "e.g. 'unhappy clients', 'payroll problems', 'integration running late'.")]
    public string SearchConsultations(
        [Description("Question or keywords.")] string query,
        [Description("Where to search: all, consultations, meetings, feedback or orders.")] string area = "all",
        [Description("Maximum number of results (1-50).")] int topK = 8,
        [Description("Minimum score (0-1). Omit for the recommended value.")] double? minScore = null,
        [Description("'semantic' (default), 'keyword' or 'pattern' (the query is a regex).")] string mode = "semantic")
    {
        var sources = (area ?? "all").Trim().ToLowerInvariant() switch
        {
            "all" or "" => "consultations,meetings,feedback,orders",
            "consultations" or "meetings" or "feedback" or "orders" => area!.Trim().ToLowerInvariant(),
            _ => throw new McpException("area must be one of: all, consultations, meetings, feedback, orders."),
        };
        return Run(() => engine.Search(query, topK, minScore ?? SearchEngine.RecommendedMinScore, sources, mode));
    }

    // ================================================================================== lists

    [McpServerTool(Name = "list_consultations", Title = "List consultations", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Lists consultations, newest first, filtered by client, consultant, ERP module, status, type, priority and date. " +
                 "Statuses: Requested, Scheduled, In progress, Completed, Cancelled.")]
    public string ListConsultations(
        [Description("Part of the client name.")] string? client = null,
        [Description("Part of the consultant name.")] string? consultant = null,
        [Description("ERP module: Finance, Inventory, Sales, Purchasing, HR & Payroll, Manufacturing, CRM, Reporting, Integration, Projects.")] string? module = null,
        [Description("Status: Requested, Scheduled, In progress, Completed or Cancelled.")] string? status = null,
        [Description("Type: Implementation, Training, Support, Health check, Upgrade, Data migration, Customisation.")] string? type = null,
        [Description("Priority: Low, Normal, High or Urgent.")] string? priority = null,
        [Description("Scheduled (or requested) on or after this date. " + DateHelp)] string? from = null,
        [Description("Scheduled (or requested) up to this date. " + DateHelp)] string? to = null,
        [Description("Maximum rows (1-500).")] int limit = 50) =>
        Run(() => List("consultations", erp.ListConsultations(client, consultant, module, status, type, priority, DateRange.Parse(from, to), limit)));

    [McpServerTool(Name = "get_consultation", Title = "Get consultation", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Everything about one consultation: details, client and consultant contacts, meetings, orders and feedback.")]
    public string GetConsultation([Description("Consultation id.")] int id) =>
        Run(() => erp.GetConsultation(id) ?? throw new ArgumentException($"There is no consultation {id}."));

    [McpServerTool(Name = "list_clients", Title = "List clients", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Lists client companies with their ERP system and how many consultations they have (and how many are open).")]
    public string ListClients(
        [Description("Optional text to look for in the name, industry, country or ERP system.")] string? search = null,
        [Description("Maximum rows (1-500).")] int limit = 100) =>
        Run(() => List("clients", erp.ListClients(search, limit)));

    [McpServerTool(Name = "list_consultants", Title = "List consultants", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Lists consultants with their speciality, hourly rate and number of open consultations.")]
    public string ListConsultants([Description("Only consultants who are still active.")] bool activeOnly = true) =>
        Run(() => List("consultants", erp.ListConsultants(activeOnly)));

    [McpServerTool(Name = "list_meetings", Title = "List meetings", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Lists consultation meetings with agenda, notes and action items. Use upcomingOnly for the diary.")]
    public string ListMeetings(
        [Description("Only meetings of this consultation.")] int? consultationId = null,
        [Description("Part of the client name.")] string? client = null,
        [Description("Part of the consultant name.")] string? consultant = null,
        [Description("On or after this date. " + DateHelp)] string? from = null,
        [Description("Up to this date. " + DateHelp)] string? to = null,
        [Description("Only meetings that have not happened yet (soonest first).")] bool upcomingOnly = false,
        [Description("Maximum rows (1-500).")] int limit = 50) =>
        Run(() => List("meetings", erp.ListMeetings(consultationId, client, consultant, DateRange.Parse(from, to), upcomingOnly, limit)));

    [McpServerTool(Name = "list_orders", Title = "List orders", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Lists quotes and orders, newest first. Statuses: Quoted, Confirmed, Invoiced, Paid, Cancelled.")]
    public string ListOrders(
        [Description("Part of the client name.")] string? client = null,
        [Description("Status: Quoted, Confirmed, Invoiced, Paid or Cancelled.")] string? status = null,
        [Description("Ordered on or after this date. " + DateHelp)] string? from = null,
        [Description("Ordered up to this date. " + DateHelp)] string? to = null,
        [Description("Maximum rows (1-500).")] int limit = 50) =>
        Run(() => List("orders", erp.ListOrders(client, status, DateRange.Parse(from, to), limit)));

    [McpServerTool(Name = "list_feedback", Title = "List feedback", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Lists client feedback with ratings (1-5). Filter e.g. maxRating 2 for unhappy clients, or followUpOnly.")]
    public string ListFeedback(
        [Description("Part of the client name.")] string? client = null,
        [Description("Part of the consultant name.")] string? consultant = null,
        [Description("Lowest rating to include (1-5).")] int? minRating = null,
        [Description("Highest rating to include (1-5).")] int? maxRating = null,
        [Description("Only feedback that needs a follow-up.")] bool followUpOnly = false,
        [Description("Submitted on or after this date. " + DateHelp)] string? from = null,
        [Description("Submitted up to this date. " + DateHelp)] string? to = null,
        [Description("Maximum rows (1-500).")] int limit = 50) =>
        Run(() => List("feedback", erp.ListFeedback(client, consultant, minRating, maxRating, followUpOnly, DateRange.Parse(from, to), limit)));

    [McpServerTool(Name = "erp_options", Title = "ERP options", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("The allowed values for modules, consultation types, statuses, priorities, meeting modes and report groupings, " +
                 "and which status can follow which.")]
    public string ErpOptions() => JsonSerializer.Serialize(new
    {
        modules = ErpCatalog.Modules,
        consultation_types = ErpCatalog.Types,
        consultation_statuses = ErpCatalog.ConsultationStatuses,
        consultation_status_flow = ErpCatalog.ConsultationFlow,
        priorities = ErpCatalog.Priorities,
        meeting_modes = ErpCatalog.MeetingModes,
        order_statuses = ErpCatalog.OrderStatuses,
        order_status_flow = ErpCatalog.OrderFlow,
        consultation_report_groups = ErpRepository.ConsultationGroups.Keys,
        revenue_report_groups = ErpRepository.RevenueGroups.Keys,
        feedback_report_groups = ErpRepository.FeedbackGroups.Keys,
    }, Json);

    // ================================================================================== reports

    [McpServerTool(Name = "consultation_report", Title = "Consultation report", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Counts consultations (open, completed, cancelled, urgent) and planned hours, grouped by " +
                 "status, module, type, priority, consultant, client or month.")]
    public string ConsultationReport(
        [Description("Group by: status, module, type, priority, consultant, client or month.")] string groupBy = "status",
        [Description("From date. " + DateHelp)] string? from = null,
        [Description("To date. " + DateHelp)] string? to = null) =>
        Run(() => Report(groupBy, from, to, erp.ConsultationReport(groupBy, DateRange.Parse(from, to)),
            "consultations", "open_count", "completed", "cancelled", "urgent", "planned_hours"));

    [McpServerTool(Name = "revenue_report", Title = "Revenue report", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Order totals per currency: total, paid, outstanding (confirmed or invoiced) and quoted, grouped by " +
                 "client, month, service, status, module or consultant.")]
    public string RevenueReport(
        [Description("Group by: client, month, service, status, module or consultant.")] string groupBy = "client",
        [Description("From date. " + DateHelp)] string? from = null,
        [Description("To date. " + DateHelp)] string? to = null,
        [Description("Include cancelled orders.")] bool includeCancelled = false) =>
        Run(() =>
        {
            var rows = erp.RevenueReport(groupBy, DateRange.Parse(from, to), includeCancelled);
            // Amounts in different currencies can't be added together, so totals are per currency.
            var totals = rows.GroupBy(r => (string)r["currency"]!)
                .ToDictionary(g => g.Key, g => new
                {
                    orders = g.Sum(r => Convert.ToInt32(r["orders"])),
                    hours = g.Sum(r => Convert.ToDecimal(r["hours"])),
                    total = g.Sum(r => Convert.ToDecimal(r["total"])),
                    paid = g.Sum(r => Convert.ToDecimal(r["paid"])),
                    outstanding = g.Sum(r => Convert.ToDecimal(r["outstanding"])),
                    quoted = g.Sum(r => Convert.ToDecimal(r["quoted"])),
                });
            return new { group_by = groupBy, from, to, rows, totals_by_currency = totals };
        });

    [McpServerTool(Name = "feedback_report", Title = "Feedback report", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Client satisfaction: number of responses, average and lowest rating, % who would recommend and " +
                 "follow-ups needed, grouped by consultant, client, module, type or month.")]
    public string FeedbackReport(
        [Description("Group by: consultant, client, module, type or month.")] string groupBy = "consultant",
        [Description("From date. " + DateHelp)] string? from = null,
        [Description("To date. " + DateHelp)] string? to = null) =>
        Run(() => Report(groupBy, from, to, erp.FeedbackReport(groupBy, DateRange.Parse(from, to)), "responses", "follow_ups"));

    [McpServerTool(Name = "consultant_workload", Title = "Consultant workload", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("For each consultant: consultations (open and completed), planned hours, meetings and meeting hours, " +
                 "and average client rating, optionally within a date range.")]
    public string ConsultantWorkload(
        [Description("From date. " + DateHelp)] string? from = null,
        [Description("To date. " + DateHelp)] string? to = null) =>
        Run(() => Report("consultant", from, to, erp.ConsultantWorkload(DateRange.Parse(from, to)),
            "consultations", "open_count", "completed", "planned_hours", "meetings", "meeting_hours"));

    // ================================================================================== changes

    [McpServerTool(Name = "create_consultation", Title = "Book a consultation", ReadOnly = false, Destructive = false, Idempotent = false, OpenWorld = false)]
    [Description("Books a new consultation for a client. With scheduledStart it is 'Scheduled', otherwise 'Requested'. " +
                 "Client and consultant can be given by id or (part of) their name.")]
    public async Task<string> CreateConsultation(
        [Description("Client id or name.")] string client,
        [Description("Short title, e.g. 'Payroll not calculating overtime'.")] string title,
        [Description("ERP module: Finance, Inventory, Sales, Purchasing, HR & Payroll, Manufacturing, CRM, Reporting, Integration, Projects.")] string module,
        [Description("Type: Implementation, Training, Support, Health check, Upgrade, Data migration, Customisation.")] string type,
        [Description("What the client needs.")] string description,
        [Description("Consultant id or name (optional).")] string? consultant = null,
        [Description("Low, Normal, High or Urgent.")] string priority = "Normal",
        [Description("When it starts. " + DateHelp)] string? scheduledStart = null,
        [Description("Planned hours, e.g. 4 or 1.5.")] decimal? durationHours = null,
        [Description(ByHelp)] string? performedBy = null)
    {
        return await Change(async () =>
        {
            var id = erp.CreateConsultation(client, consultant, title, module, type, description, priority,
                DateRange.ParseDate(scheduledStart, "scheduledStart"), durationHours, performedBy);
            logger.LogInformation("Consultation {Id} created by {By}", id, performedBy ?? "mcp-client");
            await Reindex();
            return new { message = $"Consultation {id} created.", consultation = erp.GetConsultation(id) };
        });
    }

    [McpServerTool(Name = "update_consultation", Title = "Update a consultation", ReadOnly = false, Destructive = false, Idempotent = false, OpenWorld = false)]
    [Description("Changes a consultation's status, consultant, schedule, hours, priority or outcome. Status rules: " +
                 "Requested -> Scheduled/In progress/Cancelled; Scheduled -> In progress/Completed/Cancelled/Requested; " +
                 "In progress -> Completed/Cancelled; Cancelled -> Requested (re-open); Completed is final. " +
                 "Completing needs an outcome; scheduling needs a scheduledStart.")]
    public async Task<string> UpdateConsultation(
        [Description("Consultation id.")] int id,
        [Description("New status.")] string? status = null,
        [Description("Consultant id or name.")] string? consultant = null,
        [Description("New start. " + DateHelp)] string? scheduledStart = null,
        [Description("Planned hours.")] decimal? durationHours = null,
        [Description("Low, Normal, High or Urgent.")] string? priority = null,
        [Description("What was achieved (needed when completing).")] string? outcome = null,
        [Description(ByHelp)] string? performedBy = null)
    {
        return await Change(async () =>
        {
            erp.UpdateConsultation(id, status, consultant, DateRange.ParseDate(scheduledStart, "scheduledStart"),
                durationHours, priority, outcome, performedBy);
            logger.LogInformation("Consultation {Id} updated by {By}", id, performedBy ?? "mcp-client");
            await Reindex();
            return new { message = $"Consultation {id} updated.", consultation = erp.GetConsultation(id) };
        });
    }

    [McpServerTool(Name = "add_meeting", Title = "Add a meeting", ReadOnly = false, Destructive = false, Idempotent = false, OpenWorld = false)]
    [Description("Records a meeting (past or planned) for a consultation, with agenda, notes and action items.")]
    public async Task<string> AddMeeting(
        [Description("Consultation id.")] int consultationId,
        [Description("When. " + DateHelp)] string meetingAt,
        [Description("What the meeting is about.")] string agenda,
        [Description("Length in minutes (5-600).")] int durationMinutes = 60,
        [Description("Online, On-site or Phone.")] string mode = "Online",
        [Description("Who attends.")] string? attendees = null,
        [Description("What was discussed.")] string? notes = null,
        [Description("Agreed next steps.")] string? actionItems = null,
        [Description(ByHelp)] string? performedBy = null)
    {
        return await Change(async () =>
        {
            var at = DateRange.ParseDate(meetingAt, "meetingAt") ?? throw new ArgumentException("meetingAt is required.");
            var id = erp.AddMeeting(consultationId, at, durationMinutes, mode, attendees, agenda, notes, actionItems, performedBy);
            await Reindex();
            return new { message = $"Meeting {id} added to consultation {consultationId}.", meeting_id = id };
        });
    }

    [McpServerTool(Name = "create_order", Title = "Create an order", ReadOnly = false, Destructive = false, Idempotent = false, OpenWorld = false)]
    [Description("Creates a quote or order (number ORD-xxxxx). Link it to a consultation, or give a client. " +
                 "Without an amount, a USD price is calculated from the consultant's hourly rate.")]
    public async Task<string> CreateOrder(
        [Description("What is being sold, e.g. 'Support hours - Finance'.")] string service,
        [Description("Number of hours.")] decimal hours,
        [Description("Consultation id this order is for (optional).")] int? consultationId = null,
        [Description("Client id or name (optional when consultationId is given).")] string? client = null,
        [Description("Price. Omit to calculate from the consultant's rate (USD only).")] decimal? amount = null,
        [Description("3-letter currency, e.g. USD or LKR.")] string currency = "USD",
        [Description("Quoted or Confirmed.")] string status = "Quoted",
        [Description("Notes.")] string? notes = null,
        [Description(ByHelp)] string? performedBy = null)
    {
        return await Change(async () =>
        {
            var (id, number) = erp.CreateOrder(client, consultationId, service, hours, amount, currency, status, notes, performedBy);
            await Reindex();
            return new { message = $"Order {number} created.", order_id = id, order_number = number };
        });
    }

    [McpServerTool(Name = "update_order_status", Title = "Change order status", ReadOnly = false, Destructive = false, Idempotent = false, OpenWorld = false)]
    [Description("Moves an order along: Quoted -> Confirmed -> Invoiced -> Paid (or Cancelled before it is paid). " +
                 "The order can be given by id or by number (ORD-00012).")]
    public async Task<string> UpdateOrderStatus(
        [Description("Order id or order number.")] string order,
        [Description("New status: Confirmed, Invoiced, Paid or Cancelled.")] string status,
        [Description("Note to add, e.g. 'Paid by bank transfer'.")] string? notes = null,
        [Description(ByHelp)] string? performedBy = null)
    {
        return await Change(async () =>
        {
            var (id, number, fromStatus, toStatus) = erp.UpdateOrderStatus(order, status, notes, performedBy);
            await Reindex();
            return new { message = $"Order {number} changed from {fromStatus} to {toStatus}.", order_id = id };
        });
    }

    [McpServerTool(Name = "add_feedback", Title = "Add client feedback", ReadOnly = false, Destructive = false, Idempotent = false, OpenWorld = false)]
    [Description("Records what a client thought of a consultation (in progress or completed): rating 1-5 and comments.")]
    public async Task<string> AddFeedback(
        [Description("Consultation id.")] int consultationId,
        [Description("Rating from 1 (very poor) to 5 (excellent).")] int rating,
        [Description("The client's comments.")] string comments,
        [Description("Would the client recommend us?")] bool wouldRecommend = true,
        [Description("Does someone need to follow up?")] bool followUpRequired = false,
        [Description(ByHelp)] string? performedBy = null)
    {
        return await Change(async () =>
        {
            var id = erp.AddFeedback(consultationId, rating, comments, wouldRecommend, followUpRequired, performedBy);
            await Reindex();
            return new { message = $"Feedback {id} saved for consultation {consultationId}.", feedback_id = id };
        });
    }

    [McpServerTool(Name = "add_client", Title = "Add a client", ReadOnly = false, Destructive = false, Idempotent = false, OpenWorld = false)]
    [Description("Adds a new client company.")]
    public async Task<string> AddClient(
        [Description("Company name (must be unique).")] string name,
        [Description("Industry, e.g. Retail.")] string industry,
        [Description("Country.")] string country,
        [Description("ERP system they use, e.g. Odoo or SAP Business One.")] string erpSystem,
        [Description("Main contact person.")] string contactName,
        [Description("Contact e-mail address.")] string contactEmail,
        [Description(ByHelp)] string? performedBy = null)
    {
        return await Change(async () =>
        {
            var id = erp.AddClient(name, industry, country, erpSystem, contactName, contactEmail, performedBy);
            await Task.CompletedTask;
            return new { message = $"Client {id} added.", client_id = id };
        });
    }

    // ================================================================================== helpers

    private static object List(string name, List<Dictionary<string, object?>> rows) =>
        new Dictionary<string, object> { ["count"] = rows.Count, [name] = rows };

    /// <summary>Report rows plus a totals row for the numeric columns that can be added up.</summary>
    private static object Report(string groupBy, string? from, string? to, List<Dictionary<string, object?>> rows, params string[] summable)
    {
        var totals = summable.ToDictionary(col => col, col => rows.Sum(r => r[col] is null ? 0m : Convert.ToDecimal(r[col])));
        return new { group_by = groupBy, from, to, rows, totals };
    }

    private static string Run(Func<object> work)
    {
        try
        {
            return JsonSerializer.Serialize(work(), Json);
        }
        catch (ArgumentException ex)
        {
            throw new McpException(ex.Message);
        }
    }

    private static async Task<string> Change(Func<Task<object>> work)
    {
        try
        {
            return JsonSerializer.Serialize(await work(), Json);
        }
        catch (ArgumentException ex)
        {
            throw new McpException(ex.Message);
        }
    }

    /// <summary>Keep search results up to date after a change. A failure here must not undo the change.</summary>
    private async Task Reindex()
    {
        try
        {
            await indexer.RebuildAsync(CancellationToken.None);
        }
        catch (Exception ex)
        {
            logger.LogWarning(ex, "Saved, but the search index could not be refreshed: {Message}", ex.Message);
        }
    }
}
