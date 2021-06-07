using System.Collections.Generic;
using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class GetRemindersQuery : IDatabaseQuery<GetRemindersQuery, IEnumerable<Reminder>>
    {
        public long UserId { get; }

        public GetRemindersQuery(long userId)
        {
            UserId = userId;
        }
    }
}