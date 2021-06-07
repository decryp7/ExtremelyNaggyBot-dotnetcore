using System;
using System.Data.SQLite;
using System.Globalization;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class GetReminderQueryHandler : SQLiteDatabaseQueryHandlerBase<GetReminderQuery, Reminder>
    {
        public override Task<DataModel.Reminder> Handle(SQLiteConnection connection, GetReminderQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                Reminder reminder = null;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "select rowid, * from reminders where row_id = @row_id";
                    command.Parameters.Add(new SQLiteParameter("row_id", databaseQuery.ReminderId));

                    using (SQLiteDataReader dataReader = command.ExecuteReader())
                    {
                        if (!dataReader.Read())
                        {
                            return reminder;
                        }

                        Recurring recurring = (Recurring) Enum.Parse(typeof(Recurring),
                            dataReader["recurring"].ToString(), true);

                        DateTime dateTime = DateTime.ParseExact(dataReader["datetime"].ToString(), "s",
                            CultureInfo.InvariantCulture);

                        reminder = new Reminder((long) dataReader["rowid"], (long) dataReader["user_id"],
                            (string) dataReader["description"], dateTime, recurring);
                    }
                }

                return reminder;
            });
        }
    }
}