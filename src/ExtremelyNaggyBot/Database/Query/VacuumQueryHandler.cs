using System;
using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query
{
    public class VacuumQueryHandler : SQLiteDatabaseQueryHandlerBase<VacuumQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, VacuumQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                try
                {
                    using (SQLiteCommand command = connection.CreateCommand())
                    {
                        if (connection.AutoCommit)
                        {
                            command.CommandText = "VACUUM;";
                            command.ExecuteNonQuery();
                            Console.WriteLine($"{DateTime.Now} Vacuumed!");
                        }
                        else
                        {
                            Console.WriteLine($"{DateTime.Now} Database has transaction pending. Skipping vacuum!");
                        }
                    }
                }
                catch (SQLiteException ex)
                {
                    Console.WriteLine($"Error occurred when vacuuming database! {ex}");
                    return false;
                }

                return true;
            });
        }
    }
}