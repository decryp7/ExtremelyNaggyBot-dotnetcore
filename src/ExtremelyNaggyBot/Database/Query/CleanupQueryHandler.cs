using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.Globalization;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query
{
    public class CleanupQueryHandler : SQLiteDatabaseQueryHandlerBase<CleanupQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, CleanupQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                try
                {
                    using (SQLiteTransaction transaction = connection.BeginTransaction())
                    using (SQLiteCommand command = connection.CreateCommand())
                    {
                        DateTime now = DateTime.Now.ToUniversalTime();

                        //Get list of reminders which has expired
                        command.CommandText = "select * from reminders where datetime(datetime) < datetime(@now)";
                        command.Parameters.Add(new SQLiteParameter("now", now.ToString("s")));

                        IList<Reminder> reminders = new List<Reminder>();
                        using (SQLiteDataReader dataReader = command.ExecuteReader())
                        {
                            while (dataReader.Read())
                            {
                                Recurring recurring = (Recurring) Enum.Parse(typeof(Recurring),
                                    dataReader["recurring"].ToString(), true);

                                DateTime dateTime = DateTime.ParseExact(dataReader["datetime"].ToString(), "s",
                                    CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal);

                                reminders.Add(new Reminder((long) dataReader["reminder_id"],
                                    (long) dataReader["user_id"],
                                    (string) dataReader["description"], dateTime, recurring));
                            }
                        }

                        //remove reminders and readd reminders with updated datetime
                        foreach (Reminder reminder in reminders)
                        {
                            command.CommandText = "delete from reminders where reminder_id = @reminder_id";
                            command.Parameters.Add(new SQLiteParameter("reminder_id", reminder.Id));
                            command.ExecuteNonQuery();

                            if (reminder.Recurring != Recurring.Once)
                            {
                                DateTime updatedDateTime = reminder.DateTime.GetNextDateTime(reminder.Recurring);
                                while (updatedDateTime < now)
                                {
                                    updatedDateTime = now.GetNextDateTime(reminder.Recurring);
                                }

                                if (reminder.Recurring != Recurring.Once)
                                {
                                    command.CommandText =
                                        "insert into reminders(user_id, description, datetime, recurring) values (@user_id, @description, @datetime, @recurring)";
                                    command.Parameters.Add(new SQLiteParameter("user_id", reminder.UserId));
                                    command.Parameters.Add(new SQLiteParameter("description", reminder.Description));
                                    command.Parameters.Add(new SQLiteParameter("datetime",
                                        updatedDateTime.ToString("s")));
                                    command.Parameters.Add(new SQLiteParameter("recurring", (uint) reminder.Recurring));
                                    command.ExecuteNonQuery();
                                }
                            }
                        }

                        //remove all naggings
                        command.CommandText = "delete from naggings";
                        command.ExecuteNonQuery();

                        transaction.Commit();
                    }
                }
                catch (SQLiteException ex)
                {
                    Console.WriteLine($"Error occurred when cleaning up the database! {ex}");
                    return false;
                }

                return true;
            });
        }
    }
}