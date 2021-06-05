using System.Collections.Generic;
using System.Data.SQLite;
using GuardLibrary;

namespace ExtremelyNaggyBot.Database
{
    public abstract class SQLiteDatabase : Database<SQLiteConnection>
    {
        private readonly string filePath;
        private SQLiteConnection connection;
        private bool isInitialized;
        private readonly object gate;

        protected SQLiteDatabase(string filePath, IEnumerable<IDatabaseQueryHandler> dbQueryhandlers) : base(dbQueryhandlers)
        {
            Guard.Ensure(filePath, nameof(filePath)).IsNotNullOrEmpty();
            this.filePath = filePath;
        }

        public override SQLiteConnection Connection
        {
            get
            {
                lock (gate)
                {
                    if (isInitialized)
                    {
                        return connection;
                    }

                    Connect();
                    isInitialized = true;

                    return connection;
                }
            }
        }

        private void Connect()
        {
            SQLiteConnectionStringBuilder connectionStringBuilder = new SQLiteConnectionStringBuilder
            {
                DataSource = filePath,
                //Refer to https://blog.devart.com/increasing-sqlite-performance.html
                JournalMode = SQLiteJournalModeEnum.Memory,
                Version = 3,
                PageSize = 4096,
                //Keep 200mb of memory cache, if db exceed 200mb, sqlite start to write to temp file.
                //200,000,000 / 4096 = 48828
                CacheSize = 48828,
                SyncMode = SynchronizationModes.Off
            };
            connectionStringBuilder.Add("PRAGMA locking_mode", "EXCLUSIVE");

            string connectionString = connectionStringBuilder.ToString();
            connection = new SQLiteConnection(connectionString);
            connection.Open();
            connection.DisposeWith(this);
        }
    }
}