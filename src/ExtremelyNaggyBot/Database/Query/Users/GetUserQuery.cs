using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Users
{
    public class GetUserQuery : IDatabaseQuery<GetUserQuery, BotUser>
    {
        public long UserId { get; }

        public GetUserQuery(long userId)
        {
            UserId = userId;
        }
    }
}