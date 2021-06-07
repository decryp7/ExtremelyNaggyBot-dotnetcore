using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class RemoveReminderQueryHandler : SQLiteDatabaseQueryHandlerBase<RemoveReminderQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, RemoveReminderQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "delete from reminders where rowid = @rowid";
                    command.Parameters.Add(new SQLiteParameter("rowid", databaseQuery.ReminderId));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}