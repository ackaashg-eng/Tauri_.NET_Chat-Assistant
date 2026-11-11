using System.ComponentModel;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using ModelContextProtocol;
using ModelContextProtocol.Server;
using SemanticSearch.McpServer.Data;
using SemanticSearch.McpServer.Search;

namespace SemanticSearch.McpServer.Tools;

/// <summary>
/// The MCP "menu": every public method marked [McpServerTool] becomes a tool that an
/// MCP client can discover with tools/list and run with tools/call. Each tool returns
/// JSON text so any client (our Rust client, Claude Desktop, MCP Inspector...) can read it.
/// </summary>
[McpServerToolType]
public sealed class SearchTools(
    SearchEngine engine,
    RecordRepository repository,
    Indexer indexer,
    RuleIndex index,
    Database database,
    ILogger<SearchTools> logger)
{
    internal static readonly JsonSerializerOptions Json = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DictionaryKeyPolicy = null,
        WriteIndented = false,
    };

    [McpServerTool(Name = "semantic_search", Title = "Semantic search", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Searches the SQL Server database by meaning using regular-expression rules (no AI model, no vectors). " +
                 "Concept rules recognise what the question is about (e.g. 'never showed up' = delivery problem) and " +
                 "rows sharing those concepts and words score highest. Mode 'pattern' treats the query as a regex.")]
    public string SemanticSearch(
        [Description("Natural-language question, keywords, or (in pattern mode) a regular expression.")] string query,
        [Description("Maximum number of results (1-50). Default 5.")] int topK = 5,
        [Description("Minimum score (0-1) a result must reach. Omit for the recommended value. Ignored in pattern mode.")] double? minScore = null,
        [Description("Optional source to search in, e.g. 'products' or 'articles'. Omit to search all.")] string? source = null,
        [Description("'semantic' (concepts + words, default), 'keyword' (words only) or 'pattern' (regex).")] string mode = "semantic")
    {
        try
        {
            var response = engine.Search(query, topK, minScore ?? SearchEngine.RecommendedMinScore, source, mode);
            logger.LogInformation("semantic_search [{Mode}] '{Query}' -> {Count} results in {Ms} ms",
                response.Mode, Truncate(query), response.Results.Count, response.TookMs);
            return JsonSerializer.Serialize(response, Json);
        }
        catch (ArgumentException ex)
        {
            throw new McpException(ex.Message);
        }
    }

    [McpServerTool(Name = "explain_query", Title = "Explain query", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Shows how the regex rules read a question: which words are compared (after stemming and synonyms) " +
                 "and which concepts were recognised, with the pattern that matched. Useful when writing rules.")]
    public string ExplainQuery([Description("The question to analyse.")] string query) =>
        JsonSerializer.Serialize(engine.Explain(query), Json);

    [McpServerTool(Name = "get_record", Title = "Get record", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Returns every column of one record, identified by its source name and id.")]
    public string GetRecord(
        [Description("Source name, e.g. 'products'.")] string source,
        [Description("The record id.")] long id)
    {
        var def = repository.FindSource(source)
                  ?? throw new McpException($"Unknown source '{source}'.");
        var row = repository.Get(def, id)
                  ?? throw new McpException($"No record {id} in '{def.Name}'.");
        return JsonSerializer.Serialize(new
        {
            source = def.Name,
            display_name = def.DisplayName,
            id,
            title = Convert.ToString(row.GetValueOrDefault(def.TitleColumn)),
            fields = row,
        }, Json);
    }

    [McpServerTool(Name = "list_sources", Title = "List sources", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Lists the searchable tables (sources) with descriptions, row counts and columns.")]
    public string ListSources()
    {
        var sources = repository.Sources.Select(s => new
        {
            name = s.Name,
            display_name = s.DisplayName,
            description = s.Description,
            table = s.Table,
            records = repository.Count(s),
            text_columns = s.TextColumns,
            display_columns = s.DisplayColumns,
        });
        return JsonSerializer.Serialize(new { sources }, Json);
    }

    [McpServerTool(Name = "index_status", Title = "Index status", ReadOnly = true, Idempotent = true, OpenWorld = false)]
    [Description("Reports the search method, how many rows and concept rules are loaded, the database server and when indexing last ran.")]
    public string IndexStatus()
    {
        var snapshot = index.Current;
        return JsonSerializer.Serialize(new
        {
            method = "regex rules",
            records = snapshot.Records.Count,
            concepts = snapshot.Analyzer.Rules.Concepts.Count,
            vocabulary = snapshot.Vocabulary,
            rules_file = snapshot.Analyzer.Rules.SourcePath,
            database = database.ServerDescription,
            last_indexed = snapshot.BuiltAt,
            recommended_min_score = SearchEngine.RecommendedMinScore,
        }, Json);
    }

    [McpServerTool(Name = "reindex", Title = "Rebuild index", ReadOnly = false, Destructive = false, Idempotent = true, OpenWorld = false)]
    [Description("Reloads search-rules.json and re-reads every row from SQL Server. Run after changing data or rules.")]
    public async Task<string> Reindex(CancellationToken cancellationToken = default)
    {
        try
        {
            var report = await indexer.RebuildAsync(cancellationToken);
            return JsonSerializer.Serialize(report, Json);
        }
        catch (Exception ex) when (ex is InvalidDataException or FileNotFoundException or System.Text.Json.JsonException)
        {
            // A typo in the rules file: keep the old index and explain what is wrong.
            throw new McpException($"The rules file could not be loaded, the previous index is still in use: {ex.Message}");
        }
    }

    private static string Truncate(string s) => s.Length <= 80 ? s : s[..80] + "…";
}
