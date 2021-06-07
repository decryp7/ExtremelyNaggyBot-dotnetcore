using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class RemoveReminderQuery : IDatabaseQuery<RemoveReminderQuery, bool>
    {
        public long ReminderId { get; }

        public RemoveReminderQuery(long reminderId)
        {
            ReminderId = reminderId;
        }
    }
}