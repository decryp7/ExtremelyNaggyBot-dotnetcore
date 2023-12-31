﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Text.Json;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query.Reminders;
using ExtremelyNaggyBot.Sentry;
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
                    using (SentryPerformanceMonitor.Measure("SendNaggingMessages", "Send nagging messages"))
                    {
                        IEnumerable<Nagging> naggings =
                            await Services.ExtremelyNaggyBotDB.Execute(new GetNaggingsQuery());

                        foreach (Nagging nagging in naggings)
                        {
                            if (dateTime > nagging.DateTime)
                            {
                                continue;
                            }

                            await Services.BotClient.SendTextMessageAsync(new ChatId(nagging.UserId),
                                nagging.Description,
                                replyMarkup: new InlineKeyboardMarkup(new[]
                                {
                                    InlineKeyboardButton.WithCallbackData("Acknowledge",
                                        JsonSerializer.Serialize(new ReminderAcknowledgement(nagging.ReminderId)))
                                }));
                            Console.WriteLine(
                                FormattableString.Invariant(
                                    $"Send \"{nagging.Description}\" to user \"{nagging.UserId}\"."));
                            await Services.ExtremelyNaggyBotDB.Execute(
                                new UpdateNaggingDatetimeQuery(new Nagging(nagging.Id, nagging.ReminderId,
                                    nagging.UserId,
                                    nagging.Description, DateTime.Now.AddMinutes(1))));
                            Console.WriteLine(
                                FormattableString.Invariant(
                                    $"Updated next nagging for \"{nagging.Description}\"."));
                        }
                    }
                })
                .DisposeWith(this);
        }
    }
}