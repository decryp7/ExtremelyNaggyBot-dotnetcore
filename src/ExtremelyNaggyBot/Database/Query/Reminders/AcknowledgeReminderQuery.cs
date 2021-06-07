using ExtremelyNaggyBot.Database.DataModel;
using GuardLibrary;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class AcknowledgeReminderQuery : IDatabaseQuery<AcknowledgeReminderQuery, bool>
    {
        public ReminderAcknowledgement Acknowledgement { get; }

        public AcknowledgeReminderQuery(ReminderAcknowledgement reminderAcknowledgement)
        {
            Guard.Ensure(reminderAcknowledgement, nameof(reminderAcknowledgement)).IsNotNull();

            Acknowledgement = reminderAcknowledgement;
        }
    }
}