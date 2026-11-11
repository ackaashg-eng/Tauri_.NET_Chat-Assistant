using System.Diagnostics;
using Microsoft.Extensions.Logging;
using SemanticSearch.McpServer.Configuration;
using SemanticSearch.McpServer.Data;

namespace SemanticSearch.McpServer.Search;

public sealed record IndexReport(string Method, int Records, int Concepts, int Vocabulary, string RulesFile, long TookMs);

/// <summary>
/// Builds the search index: (re)loads search-rules.json, reads every row from SQL Server,
/// and records which words and concepts each row contains. Nothing is written to the
/// database - the index lives in memory and is rebuilt at startup or on "reindex".
/// </summary>
public sealed class Indexer(
    RecordRepository repository,
    RuleIndex index,
    ServerOptions options,
    ILogger<Indexer> logger)
{
    private readonly SemaphoreSlim _lock = new(1, 1);

    public async Task<IndexReport> RebuildAsync(CancellationToken cancellationToken)
    {
        await _lock.WaitAsync(cancellationToken);
        try
        {
            return await Task.Run(() => Rebuild(cancellationToken), cancellationToken);
        }
        finally
        {
            _lock.Release();
        }
    }

    private IndexReport Rebuild(CancellationToken ct)
    {
        var sw = Stopwatch.StartNew();
        var rules = SearchRules.Load(options.RulesFile);
        var analyzer = new TextAnalyzer(rules);

        var records = new List<IndexedRecord>();
        foreach (var source in options.Sources)
        {
            foreach (var row in repository.ReadAll(source))
            {
                ct.ThrowIfCancellationRequested();
                var sentences = TextAnalyzer.Sentences(row.Text)
                    .Select(s => new IndexedSentence(s, analyzer.TermKeys(s), analyzer.Concepts(s)))
                    .ToList();
                records.Add(new IndexedRecord(
                    source.Name, row.Id, row.Title, row.Text,
                    analyzer.TermKeys(row.Text), analyzer.TermKeys(row.Title),
                    analyzer.Concepts(row.Text), analyzer.Concepts(row.Title),
                    sentences));
            }
        }

        var snapshot = new IndexSnapshot(analyzer, records);
        index.Replace(snapshot);
        sw.Stop();

        var report = new IndexReport("regex rules", records.Count, rules.Concepts.Count, snapshot.Vocabulary,
            rules.SourcePath, sw.ElapsedMilliseconds);
        logger.LogInformation("Index ready: {Records} records, {Concepts} concepts, {Vocabulary} distinct words in {Ms} ms (rules: {Rules})",
            report.Records, report.Concepts, report.Vocabulary, report.TookMs, report.RulesFile);
        return report;
    }
}
