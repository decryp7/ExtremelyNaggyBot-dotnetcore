using ExtremelyNaggyBot.Database.DataModel;
using GuardLibrary;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class AddReminderQuery : IDatabaseQuery<AddReminderQuery, bool>
    {
        public Reminder ReminderInfo { get; }

        public AddReminderQuery(Reminder reminderInfo)
        {
            Guard.Ensure(reminderInfo, nameof(reminderInfo)).IsNotNull();

            ReminderInfo = reminderInfo;
        }
    }
}