using System.Diagnostics;
using System.Text.RegularExpressions;
using SemanticSearch.McpServer.Data;

namespace SemanticSearch.McpServer.Search;

public enum SearchMode { Semantic, Keyword, Pattern }

public sealed record SearchResult(
    string Source,
    long Id,
    string Title,
    double Score,                               // 0..1, how well the row matches the question
    IReadOnlyList<string> MatchedBy,            // "concepts", "words" or "pattern"
    IReadOnlyList<string> MatchedConcepts,      // concept labels shared with the question
    IReadOnlyList<string> MatchedWords,         // question words found in the row
    string Snippet,                             // the sentence that matched best
    IReadOnlyDictionary<string, object?> Fields);

public sealed record SearchResponse(
    string Query,
    string Mode,
    string Method,
    int TopK,
    double MinScore,
    string? Source,
    IReadOnlyList<string> QueryConcepts,
    IReadOnlyList<string> QueryWords,
    int Candidates,
    long TookMs,
    IReadOnlyList<SearchResult> Results);

/// <summary>
/// Searches the database with regular expressions only - no AI model, no vectors.
///
///  semantic (default): the question and every row are tagged with concepts by regex rules,
///      e.g. "my parcel never showed up" and "My order has not arrived yet" are both a
///      "Delivery problem". Score = 60% shared concepts + 40% shared words (rare ones count more).
///  keyword : shared words only (after stemming and synonyms).
///  pattern : the query IS a regular expression, e.g.  E-\d{3}  or  (refund|money back)
/// </summary>
public sealed class SearchEngine(RuleIndex index, Database database, RecordRepository repository)
{
    private const double ConceptShare = 0.6;
    private const double BodyOnlyFactor = 0.85;   // a match in the title counts a bit more
    private const int MaxQueryLength = 500;
    private const int MaxPatternLength = 200;
    private static readonly TimeSpan PatternTimeout = TimeSpan.FromMilliseconds(100);

    public const double RecommendedMinScore = 0.25;

    public SearchResponse Search(string query, int topK, double minScore, string? sourceName, string? mode)
    {
        var sw = Stopwatch.StartNew();
        query = (query ?? "").Trim();
        if (query.Length == 0) throw new ArgumentException("The query must not be empty.");
        if (query.Length > MaxQueryLength) throw new ArgumentException($"The query is longer than {MaxQueryLength} characters.");
        topK = Math.Clamp(topK, 1, 50);
        minScore = Math.Clamp(minScore, 0.0, 1.0);

        // One source ("products"), several ("consultations,meetings") or all (empty).
        HashSet<string>? sources = null;
        if (!string.IsNullOrWhiteSpace(sourceName))
        {
            sources = new HashSet<string>(StringComparer.Ordinal);
            foreach (var part in sourceName.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
            {
                sources.Add(repository.FindSource(part)?.Name
                    ?? throw new ArgumentException(
                        $"Unknown source '{part}'. Valid sources: {string.Join(", ", repository.Sources.Select(s => s.Name))}."));
            }
        }
        var source = sources is null ? null : string.Join(",", sources);

        var snapshot = index.Current;
        var searchMode = ParseMode(mode);
        var records = snapshot.Records.Where(r => sources is null || sources.Contains(r.Source)).ToList();

        List<Scored> scored;
        IReadOnlyList<string> queryConcepts = Array.Empty<string>();
        IReadOnlyList<string> queryWords = Array.Empty<string>();

        if (searchMode == SearchMode.Pattern)
        {
            scored = PatternSearch(query, records);
        }
        else
        {
            var analyzer = snapshot.Analyzer;
            // Distinct comparison keys, remembering the word the user actually typed.
            var terms = analyzer.Terms(query).DistinctBy(t => t.Key).ToList();
            var concepts = searchMode == SearchMode.Semantic ? analyzer.Concepts(query) : new HashSet<string>();
            queryWords = terms.Select(t => t.Word).ToList();
            queryConcepts = analyzer.Rules.Concepts.Where(c => concepts.Contains(c.Name)).Select(c => c.Label).ToList();
            scored = RuleSearch(snapshot, records, terms, concepts, minScore);
        }

        var top = scored.Take(topK).ToList();
        var results = new List<SearchResult>(top.Count);
        using (var connection = database.Open())
        {
            foreach (var s in top)
            {
                var def = repository.FindSource(s.Record.Source)!;
                var row = repository.Get(def, s.Record.Id, connection);
                if (row is null) continue; // deleted since the last index run
                var fields = def.DisplayColumns.ToDictionary(col => col, col => row.GetValueOrDefault(col));
                results.Add(new SearchResult(s.Record.Source, s.Record.Id, s.Record.Title, Math.Round(s.Score, 4),
                    s.MatchedBy, s.Concepts, s.Words, s.Snippet, fields));
            }
        }

        sw.Stop();
        return new SearchResponse(query, searchMode.ToString().ToLowerInvariant(), "regex rules", topK, minScore, source,
            queryConcepts, queryWords, records.Count, sw.ElapsedMilliseconds, results);
    }

    /// <summary>What the rules see in a question - useful for writing and debugging rules.</summary>
    public object Explain(string query)
    {
        var analyzer = index.Current.Analyzer;
        var normalized = TextAnalyzer.Normalize(query ?? "");
        return new
        {
            query,
            words = analyzer.Terms(query ?? "").Select(t => new { word = t.Word, compared_as = t.Key }).ToList(),
            concepts = analyzer.Rules.Concepts
                .Select(c => new { c.Label, Pattern = c.Patterns.FirstOrDefault(p => SafeIsMatch(p, normalized))?.ToString() })
                .Where(x => x.Pattern is not null)
                .Select(x => new { concept = x.Label, matched_pattern = x.Pattern })
                .ToList(),
        };
    }

    // ------------------------------------------------------------------------------------

    private static List<Scored> RuleSearch(IndexSnapshot snapshot, List<IndexedRecord> records,
        List<Term> terms, HashSet<string> concepts, double minScore)
    {
        if (terms.Count == 0 && concepts.Count == 0) return new();

        var termTotal = terms.Sum(t => snapshot.TermWeight(t.Key));
        var conceptTotal = concepts.Sum(snapshot.ConceptWeight);
        var labels = snapshot.Analyzer.Rules.Concepts.ToDictionary(c => c.Name, c => c.Label);
        var list = new List<Scored>();

        foreach (var r in records)
        {
            var matchedTerms = terms.Where(t => r.Terms.Contains(t.Key)).ToList();
            var matchedConcepts = concepts.Where(r.Concepts.Contains).ToList();
            if (matchedTerms.Count == 0 && matchedConcepts.Count == 0) continue;

            var termScore = termTotal <= 0 ? 0 : matchedTerms.Sum(t =>
                snapshot.TermWeight(t.Key) * (r.TitleTerms.Contains(t.Key) ? 1 : BodyOnlyFactor)) / termTotal;
            var conceptScore = conceptTotal <= 0 ? 0 : matchedConcepts.Sum(c =>
                snapshot.ConceptWeight(c) * (r.TitleConcepts.Contains(c) ? 1 : BodyOnlyFactor)) / conceptTotal;

            var score = concepts.Count > 0 ? ConceptShare * conceptScore + (1 - ConceptShare) * termScore : termScore;
            if (score < minScore) continue;

            var matchedBy = new List<string>();
            if (matchedConcepts.Count > 0) matchedBy.Add("concepts");
            if (matchedTerms.Count > 0) matchedBy.Add("words");

            var termKeys = matchedTerms.Select(t => t.Key).ToHashSet();
            list.Add(new Scored(r, score, matchedBy,
                matchedConcepts.Select(c => labels[c]).ToList(),
                matchedTerms.Select(t => t.Word).ToList(),
                BestSentence(r, termKeys, matchedConcepts)));
        }

        return list.OrderByDescending(s => s.Score).ThenBy(s => s.Record.Title, StringComparer.OrdinalIgnoreCase).ToList();
    }

    private static List<Scored> PatternSearch(string pattern, List<IndexedRecord> records)
    {
        if (pattern.Length > MaxPatternLength)
            throw new ArgumentException($"Patterns can be at most {MaxPatternLength} characters.");

        Regex regex;
        try
        {
            regex = new Regex(pattern, RegexOptions.IgnoreCase | RegexOptions.CultureInvariant, PatternTimeout);
        }
        catch (ArgumentException ex)
        {
            throw new ArgumentException($"That is not a valid regular expression: {ex.Message}");
        }

        var list = new List<Scored>();
        try
        {
            foreach (var r in records)
            {
                var matches = regex.Matches(r.Text);
                if (matches.Count == 0) continue;
                var first = matches[0];
                var sentence = r.Sentences.FirstOrDefault(s => s.Text.Contains(first.Value, StringComparison.OrdinalIgnoreCase))?.Text ?? r.Text;
                list.Add(new Scored(r, 1.0, new[] { "pattern" }, Array.Empty<string>(),
                    matches.Select(m => m.Value).Where(v => v.Length > 0).Distinct(StringComparer.OrdinalIgnoreCase).Take(5).ToList(),
                    Trim(sentence)) { MatchCount = matches.Count });
            }
        }
        catch (RegexMatchTimeoutException)
        {
            throw new ArgumentException("That pattern is too complex to run quickly. Try a simpler one.");
        }

        return list.OrderByDescending(s => s.MatchCount).ThenBy(s => s.Record.Title, StringComparer.OrdinalIgnoreCase).ToList();
    }

    /// <summary>The sentence sharing the most words/concepts with the question.</summary>
    private static string BestSentence(IndexedRecord r, HashSet<string> terms, List<string> concepts)
    {
        var best = r.Sentences
            .Select(s => (s.Text, Score: s.Terms.Count(terms.Contains) + 2 * concepts.Count(s.Concepts.Contains)))
            .OrderByDescending(x => x.Score)
            .FirstOrDefault();
        return Trim(best.Text ?? r.Text);
    }

    private static string Trim(string text, int max = 280)
    {
        text = text.Trim();
        if (text.Length <= max) return text;
        var cut = text.LastIndexOf(' ', max);
        return text[..(cut > max / 2 ? cut : max)] + "…";
    }

    private static bool SafeIsMatch(Regex regex, string text)
    {
        try { return regex.IsMatch(text); }
        catch (RegexMatchTimeoutException) { return false; }
    }

    private static SearchMode ParseMode(string? mode) => (mode ?? "semantic").Trim().ToLowerInvariant() switch
    {
        "semantic" or "hybrid" or "" => SearchMode.Semantic,
        "keyword" or "words" => SearchMode.Keyword,
        "pattern" or "regex" => SearchMode.Pattern,
        _ => throw new ArgumentException($"Unknown mode '{mode}'. Use semantic, keyword or pattern."),
    };

    private sealed record Scored(
        IndexedRecord Record, double Score, IReadOnlyList<string> MatchedBy,
        IReadOnlyList<string> Concepts, IReadOnlyList<string> Words, string Snippet)
    {
        public int MatchCount { get; init; }
    }
}
