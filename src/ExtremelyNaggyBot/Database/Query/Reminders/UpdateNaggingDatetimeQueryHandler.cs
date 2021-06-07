using System.Data.SQLite;
using System.Threading.Tasks;
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
                    command.CommandText = "update naggings set datetime = @datetime where rowid = @rowid";
                    command.Parameters.Add(new SQLiteParameter("datetime",
                        databaseQuery.Nagging.DateTime.ToUniversalTime().ToString("s")));
                    command.Parameters.Add(new SQLiteParameter("rowid", databaseQuery.Nagging.Id));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}