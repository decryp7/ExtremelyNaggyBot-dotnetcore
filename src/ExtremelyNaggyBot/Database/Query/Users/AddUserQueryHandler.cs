using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Users
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
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.UserInfo.Id));
                    command.Parameters.Add(new SQLiteParameter("first_name", databaseQuery.UserInfo.FirstName));
                    command.Parameters.Add(new SQLiteParameter("last_name", databaseQuery.UserInfo.LastName));
                    command.Parameters.Add(new SQLiteParameter("timezone_offset", databaseQuery.UserInfo.TimezoneOffset));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}