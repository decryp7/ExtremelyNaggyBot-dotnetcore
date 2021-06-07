using ExtremelyNaggyBot.Database.DataModel;
using GuardLibrary;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class UpdateNaggingDatetimeQuery : IDatabaseQuery<UpdateNaggingDatetimeQuery, bool>
    {
        public Nagging Nagging { get; }

        public UpdateNaggingDatetimeQuery(Nagging nagging)
        {
            Guard.Ensure(nagging, nameof(nagging)).IsNotNull();

            Nagging = nagging;
        }
    }
}