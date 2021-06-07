using System;
using GuardLibrary;

namespace ExtremelyNaggyBot.Database.DataModel
{
    public class Nagging
    {
        public long Id { get; } = -1;

        public long ReminderId { get; }

        public string Description { get; }

        public long UserId { get; }

        public DateTime DateTime { get; }

        public Nagging(long reminderId, long userId, string description, DateTime dateTime)
        {
            Guard.Ensure(description, nameof(description)).IsNotNullOrEmpty();

            this.ReminderId = reminderId;
            this.UserId = userId;
            this.DateTime = dateTime;
            this.Description = description;
        }

        public Nagging(long id, long reminderId, long userId, string description, DateTime dateTime)
            : this(reminderId, userId, description, dateTime)
        {
            this.Id = id;
        }
    }
}