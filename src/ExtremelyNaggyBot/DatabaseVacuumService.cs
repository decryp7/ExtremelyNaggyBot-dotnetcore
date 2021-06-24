using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Text.Json;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query;
using ExtremelyNaggyBot.Database.Query.Reminders;
using ExtremelyNaggyBot.Sentry;
using Telegram.Bot.Types;
using Telegram.Bot.Types.ReplyMarkups;

namespace ExtremelyNaggyBot
{
    public class DatabaseVacuumService : DisposableObject
    {
        public DatabaseVacuumService()
        {
            Services.Clock.Tick
                .ObserveOn(Scheduler.Default)
                .Buffer(60) //every hour
                .Select(list => list.Last())
                .Subscribe(async dateTime =>
                {
                    using (SentryPerformanceMonitor.Measure("Vacuum", "Vacuum Database"))
                    {
                        await Services.ExtremelyNaggyBotDB.Execute(new VacuumQuery());
                    }
                })
                .DisposeWith(this);
        }
    }
}