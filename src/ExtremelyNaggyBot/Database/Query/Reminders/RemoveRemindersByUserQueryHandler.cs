using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class RemoveRemindersByUserQueryHandler : SQLiteDatabaseQueryHandlerBase<RemoveRemindersByUserQuery, int>
    {
        public override Task<int> Handle(SQLiteConnection connection, RemoveRemindersByUserQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "delete from reminders where user_id = @user_id";
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.UserId));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result;
            });
        }
    }
}