using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.Globalization;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class GetRemindersQueryByUserHandler : SQLiteDatabaseQueryHandlerBase<GetRemindersQueryByUser, IEnumerable<Reminder>>
    {
        public override Task<IEnumerable<Reminder>> Handle(SQLiteConnection connection, GetRemindersQueryByUser databaseQuery)
        {
            return Task.Run(() =>
            {
                IList<Reminder> reminders = new List<Reminder>();

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "select rowid, * from reminders where user_id = @user_id";
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.UserId));

                    using (SQLiteDataReader dataReader = command.ExecuteReader())
                    {  
                        while(dataReader.Read())
                        {
                            Recurring recurring = (Recurring) Enum.Parse(typeof(Recurring), dataReader["recurring"].ToString(), true);

                            DateTime dateTime = DateTime.ParseExact(dataReader["datetime"].ToString(), "s",
                                CultureInfo.InvariantCulture);

                            reminders.Add(new Reminder((long) dataReader["rowid"], (long) dataReader["user_id"],
                                (string) dataReader["description"], dateTime, recurring));
                        }
                    }
                }

                return (IEnumerable<Reminder>)reminders;
            });
        }
    }
}