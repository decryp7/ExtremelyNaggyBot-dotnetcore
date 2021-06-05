using System;
using System.IO;
using GuardLibrary;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query
{
    public class SetupQuery : IDatabaseQuery<SetupQuery, bool>
    {
        public string Script { get; }

        public SetupQuery()
        {
            string databaseScript = Path.Combine("DatabaseScripts", "1.sql");

            Guard.EnsureThisConditionIsMet(() => File.Exists(databaseScript))
                .OrThrowException(new InvalidOperationException($"{databaseScript} does not exist."));

            Script = File.ReadAllText(databaseScript);

        }
    }
}