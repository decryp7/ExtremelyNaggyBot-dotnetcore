using ExtremelyNaggyBot.Database.DataModel;
using GuardLibrary;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query
{
    public class AddUserQuery : IDatabaseQuery<AddUserQuery, bool>
    {
        public BotUser User { get; }

        public AddUserQuery(BotUser user)
        {
            Guard.Ensure(user, nameof(user)).IsNotNull();
            User = user;
        }
    }
}