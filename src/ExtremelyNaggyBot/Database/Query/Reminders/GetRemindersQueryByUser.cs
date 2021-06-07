using System.Collections.Generic;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class GetRemindersQueryByUser : IDatabaseQuery<GetRemindersQueryByUser, IEnumerable<Reminder>>
    {
        public long UserId { get; }

        public GetRemindersQueryByUser(long userId)
        {
            UserId = userId;
        }
    }
}