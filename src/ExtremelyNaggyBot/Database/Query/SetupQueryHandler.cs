using System;
using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query
{
    public class SetupQueryHandler : SQLiteDatabaseQueryHandlerBase<SetupQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, SetupQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                try
                {
                    using (SQLiteTransaction transaction = connection.BeginTransaction())
                    using (SQLiteCommand command = connection.CreateCommand())
                    {
                        command.CommandText = databaseQuery.Script;
                        command.ExecuteNonQuery();

                        transaction.Commit();
                    }
                }
                catch (SQLiteException ex)
                {
                    Console.WriteLine($"Error occurred when setting up database! {ex}");
                    return false;
                }

                return true;
            });
        }
    }
}