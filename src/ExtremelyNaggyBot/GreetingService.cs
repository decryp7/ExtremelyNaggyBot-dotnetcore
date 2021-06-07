using System;
using System.Collections.Generic;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query;
using ExtremelyNaggyBot.Database.Query.Users;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot
{
    public class GreetingService : DisposableObject
    {
        public GreetingService()
        {
            Services.Clock.Tick
                .ObserveOn(Scheduler.Default)
                .Subscribe(async dateTime =>
                {
                    IEnumerable<BotUser> botUsers = await Services.ExtremelyNaggyBotDB.Execute(new GetUsersQuery());

                    foreach (BotUser botUser in botUsers)
                    {
                        string message = String.Empty;

                        DateTime d = TimeZoneInfo.ConvertTimeFromUtc(dateTime,
                            TimeZoneInfo.CreateCustomTimeZone("test", TimeSpan.FromHours(botUser.TimezoneOffset), null, null));
                        message = d.Hour switch
                        {
                            9 when d.Minute == 0 => "Good Morning!",
                            12 when d.Minute == 0 => "Good Afternoon!",
                            15 when d.Minute == 0 => "Nap Time!",
                            21 when d.Minute == 0 => "Good Night!",
                            _ => message
                        };

                        if (string.IsNullOrEmpty(message))
                        {
                            return;
                        }

                        Services.BotClient.SendTextMessageAsync(new ChatId(botUser.Id), message);
                    }

                }).DisposeWith(this);
        }
    }
}