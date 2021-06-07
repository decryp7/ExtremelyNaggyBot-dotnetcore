using System.Collections.Generic;
using System.Data.SQLite;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query.Users
{
    public class GetUsersQueryHandler : SQLiteDatabaseQueryHandlerBase<GetUsersQuery, IEnumerable<BotUser>>
    {
        public override Task<IEnumerable<BotUser>> Handle(SQLiteConnection connection, GetUsersQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                IList<BotUser> users = new List<BotUser>();

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "select * from users";

                    using (SQLiteDataReader dataReader = command.ExecuteReader())
                    {
                        while (dataReader.Read())
                        {
                            users.Add(new BotUser((long)dataReader["user_id"], (string)dataReader["first_name"], (string)dataReader["last_name"],
                                (long)dataReader["timezone_offset"]));
                        }
                    }
                }

                return (IEnumerable<BotUser>) users;
            });
        }
    }
}