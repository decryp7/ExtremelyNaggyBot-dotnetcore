using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class RemoveRemindersByUserQuery : IDatabaseQuery<RemoveRemindersByUserQuery, int>
    {
        public long UserId { get; }

        public RemoveRemindersByUserQuery(long userId)
        {
            UserId = userId;
        }
    }
}