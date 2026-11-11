using System.Text.RegularExpressions;

namespace SemanticSearch.McpServer.Search;

/// <summary>One searchable word: the word as written, and the form used for comparing.</summary>
public readonly record struct Term(string Word, string Key);

/// <summary>
/// Turns text into comparable pieces using only regular expressions:
///   1. words      : regex  \w+(?:[-']\w+)*   ("wi-fi", "won't", "e-104" stay whole)
///   2. stop words : dropped ("the", "my", ...)
///   3. stemming   : regex suffix rules ("charging" -> "charg")
///   4. synonyms   : "package" -> "parcel"
///   5. concepts   : concept regexes run over the whole sentence ("never showed up" -> Delivery problem)
/// </summary>
public sealed partial class TextAnalyzer(SearchRules rules)
{
    [GeneratedRegex(@"\w+(?:[-']\w+)*")]
    private static partial Regex WordRegex();

    [GeneratedRegex(@"(?<=[.!?])\s+")]
    private static partial Regex SentenceRegex();

    public SearchRules Rules => rules;

    public static string Normalize(string text) =>
        text.ToLowerInvariant().Replace('’', '\'').Replace('‘', '\'');

    public List<Term> Terms(string text)
    {
        var list = new List<Term>();
        foreach (Match m in WordRegex().Matches(Normalize(text)))
        {
            var word = m.Value;
            if (word.Length < 2 || rules.StopWords.Contains(word)) continue;
            var stem = rules.Stem(word);
            list.Add(new Term(word, rules.Synonyms.TryGetValue(stem, out var canon) ? canon : stem));
        }
        return list;
    }

    public HashSet<string> TermKeys(string text) => Terms(text).Select(t => t.Key).ToHashSet(StringComparer.Ordinal);

    /// <summary>Names of every concept whose regex matches the text.</summary>
    public HashSet<string> Concepts(string text)
    {
        var normalized = Normalize(text);
        var found = new HashSet<string>(StringComparer.Ordinal);
        foreach (var concept in rules.Concepts)
        {
            try
            {
                if (concept.IsMatch(normalized)) found.Add(concept.Name);
            }
            catch (RegexMatchTimeoutException)
            {
                // A pathological rule on unusual text: treat as "no match" rather than failing the search.
            }
        }
        return found;
    }

    public static string[] Sentences(string text) =>
        SentenceRegex().Split(text.Trim()).Where(s => !string.IsNullOrWhiteSpace(s)).ToArray();
}
