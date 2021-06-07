using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class RemoveReminderHandler : SQLiteDatabaseQueryHandlerBase<RemoveReminderQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, RemoveReminderQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "delete from reminders where reminder_id = @reminder_id";
                    command.Parameters.Add(new SQLiteParameter("reminder_id", databaseQuery.ReminderId));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}