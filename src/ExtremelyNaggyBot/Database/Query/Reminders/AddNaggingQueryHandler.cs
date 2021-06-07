using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class AddNaggingQueryHandler : SQLiteDatabaseQueryHandlerBase<AddNaggingQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, AddNaggingQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "insert into naggings(reminder_id, user_id, description, datetime) values (@reminder_id, @user_id, @description, @datetime)";
                    command.Parameters.Add(new SQLiteParameter("reminder_id", databaseQuery.Nagging.ReminderId));
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.Nagging.UserId));
                    command.Parameters.Add(new SQLiteParameter("description", databaseQuery.Nagging.Description));
                    command.Parameters.Add(new SQLiteParameter("datetime", databaseQuery.Nagging.DateTime.ToUniversalTime().ToString("s")));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}