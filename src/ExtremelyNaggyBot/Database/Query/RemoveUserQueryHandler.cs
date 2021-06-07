using System.Data.SQLite;
using System.Threading.Tasks;
using SimpleDatabase.SQLite;

namespace ExtremelyNaggyBot.Database.Query
{
    public class RemoveUserQueryHandler : SQLiteDatabaseQueryHandlerBase<RemoveUserQuery, bool>
    {
        public override Task<bool> Handle(SQLiteConnection connection, RemoveUserQuery databaseQuery)
        {
            return Task.Run(() =>
            {
                int result = 0;

                using (SQLiteTransaction transaction = connection.BeginTransaction())
                using (SQLiteCommand command = connection.CreateCommand())
                {
                    command.CommandText = "delete from users where user_id = @user_id";
                    command.Parameters.Add(new SQLiteParameter("user_id", databaseQuery.UserId));

                    result += command.ExecuteNonQuery();
                    transaction.Commit();
                }

                return result != 0;
            });
        }
    }
}