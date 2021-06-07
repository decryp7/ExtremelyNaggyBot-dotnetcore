using ExtremelyNaggyBot.Database.DataModel;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class GetReminderQuery : IDatabaseQuery<GetReminderQuery, Reminder>
    {
        public long ReminderId { get; }

        public GetReminderQuery(long reminderId)
        {
            ReminderId = reminderId;
        }
    }
}