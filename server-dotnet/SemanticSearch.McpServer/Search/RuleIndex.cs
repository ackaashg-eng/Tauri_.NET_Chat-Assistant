namespace SemanticSearch.McpServer.Search;

/// <summary>A sentence of a record, pre-analysed so the best snippet can be picked quickly.</summary>
public sealed record IndexedSentence(string Text, IReadOnlySet<string> Terms, IReadOnlySet<string> Concepts);

/// <summary>One database row after analysis: its words and the concepts (meanings) it contains.</summary>
public sealed record IndexedRecord(
    string Source,
    long Id,
    string Title,
    string Text,
    IReadOnlySet<string> Terms,
    IReadOnlySet<string> TitleTerms,
    IReadOnlySet<string> Concepts,
    IReadOnlySet<string> TitleConcepts,
    IReadOnlyList<IndexedSentence> Sentences);

/// <summary>
/// A complete, read-only search index. Rebuilding creates a new snapshot and swaps it in,
/// so searches running at the same time never see a half-built index.
/// </summary>
public sealed class IndexSnapshot
{
    private readonly Dictionary<string, int> _termCounts;
    private readonly Dictionary<string, int> _conceptCounts;

    public TextAnalyzer Analyzer { get; }
    public IReadOnlyList<IndexedRecord> Records { get; }
    public DateTimeOffset BuiltAt { get; } = DateTimeOffset.UtcNow;
    public int Vocabulary => _termCounts.Count;

    public IndexSnapshot(TextAnalyzer analyzer, IReadOnlyList<IndexedRecord> records)
    {
        Analyzer = analyzer;
        Records = records;
        _termCounts = new(StringComparer.Ordinal);
        _conceptCounts = new(StringComparer.Ordinal);
        foreach (var r in records)
        {
            foreach (var t in r.Terms) _termCounts[t] = _termCounts.GetValueOrDefault(t) + 1;
            foreach (var c in r.Concepts) _conceptCounts[c] = _conceptCounts.GetValueOrDefault(c) + 1;
        }
    }

    /// <summary>
    /// "Inverse document frequency": rare words/concepts count more than common ones.
    /// A word found in every row ("product") tells us little; a rare one ("thermostat") a lot.
    /// </summary>
    public double TermWeight(string term) => Math.Log(1 + Records.Count / (double)Math.Max(0.5, _termCounts.GetValueOrDefault(term)));

    public double ConceptWeight(string concept) => Math.Log(1 + Records.Count / (double)Math.Max(0.5, _conceptCounts.GetValueOrDefault(concept)));
}

public sealed class RuleIndex
{
    private volatile IndexSnapshot? _current;

    public IndexSnapshot Current => _current ?? throw new InvalidOperationException("The search index is not built yet.");
    public bool IsReady => _current is not null;

    public void Replace(IndexSnapshot snapshot) => _current = snapshot;
}
