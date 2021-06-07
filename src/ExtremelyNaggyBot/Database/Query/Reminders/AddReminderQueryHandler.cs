using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class AddReminderQueryHandler : SQLiteDatabaseQueryHandlerBase<AddReminderQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, AddReminderQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "insert into reminders values (@user_id, @description, @datetime, @recurring)";
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.ReminderInfo.UserId));
                    command.Parameters.Add(new SQLiteParameter("description", databaseQuery.ReminderInfo.Description));
                    command.Parameters.Add(new SQLiteParameter("datetime", databaseQuery.ReminderInfo.DateTime.ToUniversalTime().ToString("s")));
                    command.Parameters.Add(new SQLiteParameter("recurring", (uint)databaseQuery.ReminderInfo.Recurring));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}