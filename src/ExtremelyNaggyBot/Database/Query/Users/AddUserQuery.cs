using ExtremelyNaggyBot.Database.DataModel;
using GuardLibrary;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Users
{
    public class AddUserQuery : IDatabaseQuery<AddUserQuery, bool>
    {
        public BotUser UserInfo { get; }

        public AddUserQuery(BotUser userInfo)
        {
            Guard.Ensure(userInfo, nameof(userInfo)).IsNotNull();
            UserInfo = userInfo;
        }
    }
}