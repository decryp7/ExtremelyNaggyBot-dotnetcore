using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.Globalization;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class GetNaggingsQueryHandler : SQLiteDatabaseQueryHandlerBase<GetNaggingsQuery, IEnumerable<Nagging>>
    {
        public override Task<IEnumerable<Nagging>> Handle(SQLiteConnection connection, GetNaggingsQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                IList<Nagging> naggings = new List<Nagging>();

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "select rowid, * from naggings";

                    using (SQLiteDataReader dataReader = command.ExecuteReader())
                    {
                        while (dataReader.Read())
                        {
                            DateTime dateTime = DateTime.ParseExact(dataReader["datetime"].ToString(), "s",
                                CultureInfo.InvariantCulture);

                            naggings.Add(new Nagging((long)dataReader["rowid"], (long)dataReader["reminder_id"],
                                (long)dataReader["user_id"], (string)dataReader["description"], dateTime));
                        }
                    }
                }

                return (IEnumerable<Nagging>) naggings;
            });
        }
    }
}