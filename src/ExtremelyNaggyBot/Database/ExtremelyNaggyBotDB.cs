using System.Collections.Generic;
using System.IO;
using SimpleDatabase;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database
{
    public class ExtremelyNaggyBotDB : SQLiteDatabaseBase
    {
        public ExtremelyNaggyBotDB(string filePath, IEnumerable<IDatabaseQueryHandler> dbQueryhandlers) 
            : base(filePath, dbQueryhandlers)
        {
            string directoryName = Path.GetDirectoryName(filePath);

            if (string.IsNullOrEmpty(directoryName))
            {
                return;
            }

            if (!Directory.Exists(directoryName))
            {
                Directory.CreateDirectory(directoryName);
            }
        }
    }
}