using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class RemoveReminderQuery : IDatabaseQuery<RemoveReminderQuery, bool>
    {
        public long ReminderId { get; }
        public long UserId { get; }

        public RemoveReminderQuery(long reminderId, long userId)
        {
            ReminderId = reminderId;
            UserId = userId;
        }
    }
}