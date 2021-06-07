using GuardLibrary;

namespace ExtremelyNaggyBot.Database.DataModel
{
    public class BotUser
    {
        public long Id { get; }

        public string FirstName { get; }

        public string LastName { get; }

        public long TimezoneOffset { get; }

        public BotUser(long id, string firstName, string lastName, long timezoneOffset)
        {
            Guard.Ensure(firstName, nameof(firstName)).IsNotNullOrEmpty();
            Guard.Ensure(lastName, nameof(lastName)).IsNotNull();

            Id = id;
            FirstName = firstName;
            LastName = lastName;
            TimezoneOffset = timezoneOffset;
        }
    }
}