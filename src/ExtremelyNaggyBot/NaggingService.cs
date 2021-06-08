using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Text.Json;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query.Reminders;
using Telegram.Bot.Types;
using Telegram.Bot.Types.ReplyMarkups;

namespace ExtremelyNaggyBot
{
    public class NaggingService : DisposableObject
    {
        public NaggingService()
        {
            Services.Clock.Tick
                .ObserveOn(Scheduler.Default)
                .Subscribe(async dateTime =>
                {
                    IEnumerable<Nagging> naggings = await Services.ExtremelyNaggyBotDB.Execute(new GetNaggingsQuery());

                    foreach (Nagging nagging in naggings)
                    {
                        if (dateTime != nagging.DateTime)
                        {
                            continue;
                        }

                        await Services.BotClient.SendTextMessageAsync(new ChatId(nagging.UserId), nagging.Description,
                            replyMarkup: new InlineKeyboardMarkup(new[]
                            {
                                InlineKeyboardButton.WithCallbackData("Acknowledge",
                                    JsonSerializer.Serialize(new ReminderAcknowledgement(nagging.ReminderId)))
                            }));
                        await Services.ExtremelyNaggyBotDB.Execute(
                            new UpdateNaggingDatetimeQuery(new Nagging(nagging.Id, nagging.ReminderId, nagging.UserId,
                                nagging.Description, dateTime.AddMinutes(1))));
                    }
                })
                .DisposeWith(this);
        }
    }
}