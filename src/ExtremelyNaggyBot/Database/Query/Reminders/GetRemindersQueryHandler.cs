using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.Globalization;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class GetRemindersQueryHandler : SQLiteDatabaseQueryHandlerBase<GetRemindersQuery, IEnumerable<Reminder>>
    {
        public override Task<IEnumerable<Reminder>> Handle(SQLiteConnection connection, GetRemindersQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                IList<Reminder> reminders = new List<Reminder>();

                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "select * from reminders";

                    using (SQLiteDataReader dataReader = command.ExecuteReader())
                    {
                        while (dataReader.Read())
                        {
                            Recurring recurring = (Recurring)Enum.Parse(typeof(Recurring), dataReader["recurring"].ToString(), true);

                            DateTime dateTime = DateTime.ParseExact(dataReader["datetime"].ToString(), "s",
                                CultureInfo.InvariantCulture);

                            reminders.Add(new Reminder((long)dataReader["reminder_id"], (long)dataReader["user_id"],
                                (string)dataReader["description"], dateTime, recurring));
                        }
                    }
                }

                return (IEnumerable<Reminder>)reminders;
            });
        }
    }
}