using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query
{
    public class AddUserQueryHandler : SQLiteDatabaseQueryHandlerBase<AddUserQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, AddUserQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "insert into users values (@user_id, @first_name, @last_name, @timezone_offset)";
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.User.Id));
                    command.Parameters.Add(new SQLiteParameter("first_name", databaseQuery.User.FirstName));
                    command.Parameters.Add(new SQLiteParameter("last_name", databaseQuery.User.LastName));
                    command.Parameters.Add(new SQLiteParameter("timezone_offset", databaseQuery.User.TimezoneOffset));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}