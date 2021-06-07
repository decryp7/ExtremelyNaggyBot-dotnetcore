using System;
using System.Data.SQLite;
using System.Globalization;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class AcknowledgeReminderQueryHandler : SQLiteDatabaseQueryHandlerBase<AcknowledgeReminderQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, AcknowledgeReminderQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    //retrieve the reminder
                    command.CommandText = "select rowid, * from reminders where row_id = @row_id";
                    command.Parameters.Add(new SQLiteParameter("row_id", databaseQuery.Acknowledgement.Id));

                    Reminder reminder = null;
                    using (SQLiteDataReader dataReader = command.ExecuteReader())
                    {
                        if (!dataReader.Read())
                        {
                            return false;
                        }

                        Recurring recurring = (Recurring)Enum.Parse(typeof(Recurring),
                            dataReader["recurring"].ToString(), true);

                        DateTime dateTime = DateTime.ParseExact(dataReader["datetime"].ToString(), "s",
                            CultureInfo.InvariantCulture);

                        reminder = new Reminder((long)dataReader["rowid"], (long)dataReader["user_id"],
                            (string)dataReader["description"], dateTime, recurring);
                    }

                    //delete the reminder
                    command.CommandText = "delete from reminders where rowid = @rowid";
                    command.Parameters.Add(new SQLiteParameter("rowid", reminder.Id));
                    result += command.ExecuteNonQuery();
                    
                    //insert the new reminder
                    if (reminder.Recurring != Recurring.Once)
                    {
                        command.CommandText =
                            "insert into reminders values (@user_id, @description, @datetime, @recurring)";
                        command.Parameters.Add(new SQLiteParameter("user_id", reminder.UserId));
                        command.Parameters.Add(new SQLiteParameter("description", reminder.Description));
                        command.Parameters.Add(new SQLiteParameter("datetime",
                            GetUpdatedDateTime(reminder.DateTime, reminder.Recurring).ToUniversalTime().ToString("s")));
                        command.Parameters.Add(new SQLiteParameter("recurring", (uint) reminder.Recurring));

                        result += command.ExecuteNonQuery();
                    }

                    transaction.Commit();
                }

                return result != 0;
            });
        }

        private DateTime GetUpdatedDateTime(DateTime dateTime, Recurring recurring)
        {
            switch (recurring)
            {
                case Recurring.Daily:
                    return dateTime.AddDays(1);
                case Recurring.Weekly:
                    return dateTime.AddDays(7);
                case Recurring.Monthly:
                    return dateTime.AddMonths(1);
            }

            return dateTime;
        }
    }
}