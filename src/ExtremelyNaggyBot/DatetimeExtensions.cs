using System;
using ExtremelyNaggyBot.Database.DataModel;

namespace ExtremelyNaggyBot
{
    public static class DatetimeExtensions
    {
        public static DateTime GetNextDateTime(this DateTime dateTime, Recurring recurring)
        {
            switch (recurring)
            {
                case Recurring.Daily:
                    return dateTime.AddDays(1);
                case Recurring.Weekly:
                    return dateTime.AddDays(7);
                case Recurring.Monthly:
                    return dateTime.AddMonths(1);
            }

            return dateTime;
        }
    }
}