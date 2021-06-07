using System.Data.SQLite;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Users
{
    public class GetUserQueryHandler : SQLiteDatabaseQueryHandlerBase<GetUserQuery, BotUser>
    {
        public override Task<BotUser> Handle(SQLiteConnection connection, GetUserQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                BotUser user = null;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "select * from users where user_id = @user_id";
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.UserId));

                    using (SQLiteDataReader dataReader = command.ExecuteReader())
                    {
                        if(dataReader.Read())
                        {
                            user = new BotUser((long)dataReader["user_id"], (string)dataReader["first_name"], (string)dataReader["last_name"],
                                (long)dataReader["timezone_offset"]);
                        }
                    }
                }

                return user;
            });
        }
    }
}