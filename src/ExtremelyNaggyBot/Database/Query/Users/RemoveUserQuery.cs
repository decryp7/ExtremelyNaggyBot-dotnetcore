using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Users
{
    public class RemoveUserQuery : IDatabaseQuery<RemoveUserQuery, bool>
    {
        public long UserId { get; }

        public RemoveUserQuery(long userId)
        {
            UserId = userId;
        }
    }
}