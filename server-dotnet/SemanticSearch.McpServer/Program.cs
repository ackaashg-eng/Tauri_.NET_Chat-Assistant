// =============================================================================
//  Semantic Search MCP Server  (C# / .NET, STDIO transport only)
//  Data: Microsoft SQL Server.   Search: regular-expression rules (no AI, no vectors).
//
//  This is a console program. It does NOT open any network port.
//  An MCP client starts it as a child process and talks JSON-RPC over:
//      stdin  -> requests from the client
//      stdout -> responses to the client   (nothing else may be written here!)
//      stderr -> logs
//
//  Usage:
//      SemanticSearch.McpServer.exe                        run as MCP server (normal)
//      SemanticSearch.McpServer.exe --index-only           create DB/tables, build the index, exit
//      SemanticSearch.McpServer.exe --connection "<SQL Server connection string>" --rules <file>
// =============================================================================

using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using SemanticSearch.McpServer.Configuration;
using SemanticSearch.McpServer.Data;
using SemanticSearch.McpServer.Search;
using SemanticSearch.McpServer.Tools;

// ---- Our own simple command-line flags (removed before the host sees args) ----
var hostArgs = new List<string>();
string? connectionOverride = null, rulesOverride = null;
var indexOnly = false;
for (var i = 0; i < args.Length; i++)
{
    switch (args[i])
    {
        case "--connection" when i + 1 < args.Length: connectionOverride = args[++i]; break;
        case "--rules" when i + 1 < args.Length: rulesOverride = args[++i]; break;
        case "--index-only": indexOnly = true; break;
        default: hostArgs.Add(args[i]); break;
    }
}

var builder = Host.CreateApplicationBuilder(new HostApplicationBuilderSettings
{
    Args = hostArgs.ToArray(),
    // appsettings.json is next to the .exe, no matter which folder the client starts us from.
    ContentRootPath = AppContext.BaseDirectory,
});
builder.Configuration.AddEnvironmentVariables(prefix: "SEMSEARCH_");

// ---- Logging: EVERYTHING goes to stderr, because stdout is the MCP channel ----
builder.Logging.ClearProviders();
builder.Logging.AddConsole(o => o.LogToStandardErrorThreshold = LogLevel.Trace);
builder.Services.Configure<ConsoleLifetimeOptions>(o => o.SuppressStatusMessages = true);

// ---- Options ----
var options = builder.Configuration.GetSection("SemanticSearch").Get<ServerOptions>() ?? new ServerOptions();
if (connectionOverride is not null) options.ConnectionString = connectionOverride;
if (rulesOverride is not null) options.RulesFile = rulesOverride;
options.Normalize(AppContext.BaseDirectory);
builder.Services.AddSingleton(options);

// ---- Application services ----
builder.Services.AddSingleton<Database>();
builder.Services.AddSingleton<RecordRepository>();
builder.Services.AddSingleton<RuleIndex>();
builder.Services.AddSingleton<Indexer>();
builder.Services.AddSingleton<SearchEngine>();
builder.Services.AddSingleton<ErpRepository>();

// ---- MCP server over STDIO ----
builder.Services
    .AddMcpServer(o =>
    {
        o.ServerInfo = new() { Name = "semantic-search", Version = "2.0.0" };
    })
    .WithStdioServerTransport()
    .WithTools<SearchTools>()
    .WithTools<ErpTools>();

using var host = builder.Build();
var log = host.Services.GetRequiredService<ILoggerFactory>().CreateLogger("Startup");

try
{
    // Create the database/tables if needed and build the search index before
    // accepting requests, so the very first search already has data.
    host.Services.GetRequiredService<Database>().EnsureCreated();
    await host.Services.GetRequiredService<Indexer>().RebuildAsync(CancellationToken.None);
}
catch (Microsoft.Data.SqlClient.SqlException ex)
{
    log.LogCritical("Could not use SQL Server: {Message}", ex.Message);
    log.LogCritical("Check the connection string (server name, e.g. localhost\\SQLEXPRESS) and that the SQL Server service is running.");
    return 1;
}
catch (Exception ex)
{
    log.LogCritical(ex, "Startup failed: {Message}", ex.Message);
    return 1;
}

if (indexOnly)
{
    log.LogInformation("--index-only: database checked and index built, exiting.");
    return 0;
}

log.LogInformation("MCP server listening on STDIO.");
await host.RunAsync();
return 0;
