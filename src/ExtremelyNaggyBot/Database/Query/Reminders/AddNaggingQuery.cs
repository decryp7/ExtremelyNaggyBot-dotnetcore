using ExtremelyNaggyBot.Database.DataModel;
using GuardLibrary;
using SimpleDatabase;

namespace ExtremelyNaggyBot.Database.Query.Reminders
{
    public class AddNaggingQuery : IDatabaseQuery<AddNaggingQuery, bool>
    {
        public Nagging Nagging { get; }

        public AddNaggingQuery(Nagging nagging)
        {
            Guard.Ensure(nagging, nameof(nagging)).IsNotNull();

            Nagging = nagging;
        }
    }
}