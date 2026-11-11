using System.Text.RegularExpressions;

namespace SemanticSearch.McpServer.Configuration;

/// <summary>Settings bound from the "SemanticSearch" section of appsettings.json.</summary>
public sealed class ServerOptions
{
    public string ConnectionString { get; set; } =
        @"Server=localhost\SQLEXPRESS;Database=LumenSearch;Trusted_Connection=True;TrustServerCertificate=True;Encrypt=True";
    public bool CreateDatabaseIfMissing { get; set; } = true;
    public bool CreateSampleData { get; set; } = true;
    public string RulesFile { get; set; } = "Rules/search-rules.json";

    /// <summary>
    /// Table/view scripts and sample rows. The schema script runs on every start (it only creates
    /// what is missing); the seed script runs only while CheckTable is empty.
    /// </summary>
    public List<SampleDataSet> SampleData { get; set; } = new();

    public List<SourceDefinition> Sources { get; set; } = new();

    /// <summary>Turns relative paths into absolute ones and validates the config.</summary>
    public void Normalize(string baseDirectory)
    {
        RulesFile = Resolve(baseDirectory, RulesFile);
        foreach (var set in SampleData)
        {
            set.SchemaScript = Resolve(baseDirectory, set.SchemaScript);
            set.SeedScript = Resolve(baseDirectory, set.SeedScript);
        }

        if (string.IsNullOrWhiteSpace(ConnectionString))
            throw new InvalidOperationException("SemanticSearch:ConnectionString is empty.");
        if (Sources.Count == 0)
            throw new InvalidOperationException("No searchable sources are configured (SemanticSearch:Sources).");

        var names = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var s in Sources)
        {
            s.Validate();
            if (!names.Add(s.Name))
                throw new InvalidOperationException($"Duplicate source name '{s.Name}'.");
        }
    }

    private static string Resolve(string baseDirectory, string path) =>
        Path.IsPathRooted(path) ? path : Path.GetFullPath(Path.Combine(baseDirectory, path));
}

public sealed class SampleDataSet
{
    public string Name { get; set; } = "";
    public string SchemaScript { get; set; } = "";
    public string SeedScript { get; set; } = "";
    public string CheckTable { get; set; } = "";
}

/// <summary>Describes one searchable table (or view).</summary>
public sealed class SourceDefinition
{
    private static readonly Regex Identifier = new("^[A-Za-z_][A-Za-z0-9_]*$", RegexOptions.Compiled);
    private static readonly Regex TableName = new(@"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)?$", RegexOptions.Compiled);

    public string Name { get; set; } = "";
    public string DisplayName { get; set; } = "";
    public string Description { get; set; } = "";
    public string Table { get; set; } = "";
    public string IdColumn { get; set; } = "id";
    public string TitleColumn { get; set; } = "";
    public List<string> TextColumns { get; set; } = new();
    public List<string> DisplayColumns { get; set; } = new();

    public void Validate()
    {
        // Table/column names are inserted into SQL text, so only plain identifiers are allowed.
        if (!TableName.IsMatch(Table))
            throw new InvalidOperationException($"Invalid table name '{Table}' in source '{Name}'.");
        foreach (var id in new[] { Name, IdColumn, TitleColumn }.Concat(TextColumns).Concat(DisplayColumns))
        {
            if (string.IsNullOrWhiteSpace(id) || !Identifier.IsMatch(id))
                throw new InvalidOperationException($"Invalid identifier '{id}' in source '{Name}'.");
        }
        if (TextColumns.Count == 0)
            throw new InvalidOperationException($"Source '{Name}' needs at least one TextColumn.");
        if (string.IsNullOrWhiteSpace(DisplayName)) DisplayName = Name;
    }

    /// <summary>"dbo.products" -> "[dbo].[products]" (safe to put in SQL text after Validate()).</summary>
    public string QuotedTable => string.Join(".", Table.Split('.').Select(Sql.Quote));
}

public static class Sql
{
    public static string Quote(string identifier) => "[" + identifier.Replace("]", "]]") + "]";
}
