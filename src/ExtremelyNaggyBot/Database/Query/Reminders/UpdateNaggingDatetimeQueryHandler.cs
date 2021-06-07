using System;
using System.Data.SQLite;
using System.Globalization;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class UpdateNaggingDatetimeQueryHandler : SQLiteDatabaseQueryHandlerBase<UpdateNaggingDatetimeQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, UpdateNaggingDatetimeQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    //retrieve the reminder
                    command.CommandText = "select * from reminders where reminder_id = @reminder_id";
                    command.Parameters.Add(new SQLiteParameter("reminder_id", databaseQuery.Nagging.ReminderId));

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

                        reminder = new Reminder((long)dataReader["reminder_id"], (long)dataReader["user_id"],
                            (string)dataReader["description"], dateTime, recurring);
                    }

                    //check if the next naggy exceed the reminder next time
                    if (reminder.Recurring != Recurring.Once && 
                        databaseQuery.Nagging.DateTime > reminder.DateTime.GetNextDateTime(reminder.Recurring))
                    {
                        //delete the nagging
                        command.CommandText = "delete from naggings where nagging_id = @nagging_id";
                        command.Parameters.Add(new SQLiteParameter("nagging_id", databaseQuery.Nagging.Id));
                        result += command.ExecuteNonQuery();
                    }
                    else
                    {
                        command.CommandText = "update naggings set datetime = @datetime where nagging_id = @nagging_id";
                        command.Parameters.Add(new SQLiteParameter("datetime",
                            databaseQuery.Nagging.DateTime.ToUniversalTime().ToString("s")));
                        command.Parameters.Add(new SQLiteParameter("nagging_id", databaseQuery.Nagging.Id));
                        result += command.ExecuteNonQuery();
                    }

                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}