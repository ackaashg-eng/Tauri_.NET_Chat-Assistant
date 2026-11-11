using System.Text.Json;
using System.Text.RegularExpressions;

namespace SemanticSearch.McpServer.Search;

/// <summary>A meaning, recognised by any of its regex patterns (e.g. "Delivery problem").</summary>
public sealed record Concept(string Name, string Label, IReadOnlyList<Regex> Patterns)
{
    public bool IsMatch(string normalizedText) => Patterns.Any(p => p.IsMatch(normalizedText));
}

/// <summary>
/// Everything that makes the search "understand" text, loaded from search-rules.json:
/// concept patterns, synonyms, stop words and stemming rules. All of it is plain regex.
/// </summary>
public sealed class SearchRules
{
    // Rules are written by the admin, but a runaway pattern must never hang the server.
    private static readonly TimeSpan RuleTimeout = TimeSpan.FromMilliseconds(250);

    public required IReadOnlyList<Concept> Concepts { get; init; }
    public required IReadOnlyDictionary<string, string> Synonyms { get; init; }   // stemmed word -> canonical word
    public required IReadOnlySet<string> StopWords { get; init; }
    public required IReadOnlyList<(Regex Pattern, string Replacement)> Stemming { get; init; }
    public required string SourcePath { get; init; }

    public static SearchRules Load(string path)
    {
        if (!File.Exists(path)) throw new FileNotFoundException($"Search rules file not found: {path}");

        using var doc = JsonDocument.Parse(File.ReadAllText(path), new JsonDocumentOptions
        {
            CommentHandling = JsonCommentHandling.Skip,
            AllowTrailingCommas = true,
        });
        var root = doc.RootElement;

        var stemming = new List<(Regex, string)>();
        if (root.TryGetProperty("stemming", out var stemEl))
            foreach (var s in stemEl.EnumerateArray())
                stemming.Add((Compile(s.GetProperty("pattern").GetString()!, "stemming"), s.GetProperty("replace").GetString() ?? ""));

        var stopWords = new HashSet<string>(StringComparer.Ordinal);
        if (root.TryGetProperty("stopWords", out var stopEl))
            foreach (var w in stopEl.EnumerateArray()) stopWords.Add(w.GetString()!.ToLowerInvariant());

        var concepts = new List<Concept>();
        if (root.TryGetProperty("concepts", out var conceptEl))
        {
            foreach (var c in conceptEl.EnumerateArray())
            {
                var name = c.GetProperty("name").GetString()!;
                var label = c.TryGetProperty("label", out var l) ? l.GetString() ?? name : name;
                var patterns = c.GetProperty("patterns").EnumerateArray()
                    .Select(p => Compile(p.GetString()!, $"concept '{name}'"))
                    .ToList();
                concepts.Add(new Concept(name, label, patterns));
            }
        }

        var rules = new SearchRules
        {
            Concepts = concepts,
            StopWords = stopWords,
            Stemming = stemming,
            Synonyms = new Dictionary<string, string>(),
            SourcePath = path,
        };

        // Synonyms are stored by their stemmed form, so "packages" and "package" both map to "parcel".
        var synonyms = new Dictionary<string, string>(StringComparer.Ordinal);
        if (root.TryGetProperty("synonyms", out var synEl))
            foreach (var group in synEl.EnumerateObject())
                foreach (var w in group.Value.EnumerateArray())
                    synonyms[rules.Stem(w.GetString()!.ToLowerInvariant())] = group.Name.ToLowerInvariant();

        return new SearchRules
        {
            Concepts = concepts, StopWords = stopWords, Stemming = stemming, Synonyms = synonyms, SourcePath = path,
        };
    }

    public string Stem(string word)
    {
        foreach (var (pattern, replacement) in Stemming) word = pattern.Replace(word, replacement);
        return word;
    }

    private static Regex Compile(string pattern, string where)
    {
        try
        {
            return new Regex(pattern, RegexOptions.CultureInvariant | RegexOptions.Compiled, RuleTimeout);
        }
        catch (ArgumentException ex)
        {
            throw new InvalidDataException($"Invalid regex in {where}: \"{pattern}\" - {ex.Message}");
        }
    }
}
