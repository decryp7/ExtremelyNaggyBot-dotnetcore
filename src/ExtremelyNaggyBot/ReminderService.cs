using System;
using System.Collections.Generic;
using System.Net.Http.Headers;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Text.Json;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query;
using ExtremelyNaggyBot.Database.Query.Reminders;
using ExtremelyNaggyBot.Database.Query.Users;
using Telegram.Bot.Types;
using Telegram.Bot.Types.ReplyMarkups;

namespace ExtremelyNaggyBot
{
    public class ReminderService : DisposableObject
    {
        public ReminderService()
        {
            Services.Clock.Tick
                .ObserveOn(Scheduler.Default)
                .Subscribe(async dateTime =>
                {
                    IEnumerable<Reminder> reminders =
                        await Services.ExtremelyNaggyBotDB.Execute(new GetRemindersQuery());

                    foreach (Reminder reminder in reminders)
                    {
                        if (dateTime != reminder.DateTime)
                        {
                            continue;
                        }

                        await Services.BotClient.SendTextMessageAsync(new ChatId(reminder.UserId), reminder.Description,
                            replyMarkup: new InlineKeyboardMarkup(new[]
                            {
                                InlineKeyboardButton.WithCallbackData("Acknowledge",
                                    JsonSerializer.Serialize(new ReminderAcknowledgement(reminder.Id)))
                            }));
                        await Services.ExtremelyNaggyBotDB.Execute(
                            new AddNaggingQuery(new Nagging(reminder.Id, reminder.UserId, reminder.Description, dateTime.AddMinutes(1))));
                    }
                })
                .DisposeWith(this);
        }
    }
}