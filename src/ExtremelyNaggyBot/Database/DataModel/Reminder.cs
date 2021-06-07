using System;
using GuardLibrary;

namespace ExtremelyNaggyBot.Database.DataModel
{
    public enum Recurring : uint
    {
        Once = 0,
        Daily = 1,
        Weekly = 2,
        Monthly = 3,
    }

    public class Reminder
    {
        public long Id { get; } = -1;

        public long UserId { get; }

        public string Description { get; }

        public DateTime DateTime { get; }

        public Recurring Recurring { get; }

        public Reminder(long userId, string description, DateTime dateTime, Recurring recurring)
        {
            Guard.Ensure(description, nameof(description)).IsNotNullOrEmpty();

            UserId = userId;
            Description = description;
            DateTime = dateTime;
            Recurring = recurring;
        }

        public Reminder(long id, long userId, string description, DateTime dateTime, Recurring recurring)
            : this(userId, description, dateTime, recurring)
        {
            Id = id;
        }
    }
}