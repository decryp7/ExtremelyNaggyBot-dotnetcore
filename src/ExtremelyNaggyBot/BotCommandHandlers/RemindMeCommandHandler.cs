﻿using System;
using System.Globalization;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query.Reminders;
using ExtremelyNaggyBot.Database.Query.Users;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class RemindMeCommandHandler : IBotCommandHandler
    {
        private readonly Regex remindmeRegex =
            new Regex(@"(.+) (\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}) (once|daily|weekly|monthly)", RegexOptions.Compiled | RegexOptions.IgnoreCase);

        public string Command { get; } = "/remindme";

        public string CommandDescription { get; } =
            "/remindme {description} {datetime:dd/MM/yyyy HH:mm} {recurring:once/daily/weekly/monthly}" + Environment.NewLine +
            "Add new reminder. " + Environment.NewLine +
            "hh:mm in 24hrs format";

        public async Task Handle(Chat chat, string commandArgs)
        {
            Match match = remindmeRegex.Match(commandArgs);

            if (match.Groups.Count != 4)
            {
                await Services.BotClient.SendTextMessageAsync(chat,
                    $"Hi {chat.FirstName}, I cannot understand your request.{Environment.NewLine}Usage: {CommandDescription}");
                return;
            }

            BotUser botUser = await Services.ExtremelyNaggyBotDB.Execute(new GetUserQuery(chat.Id));

            if (botUser == null)
            {
                await Services.BotClient.SendTextMessageAsync(chat,
                    $"Hi {chat.FirstName}, please register first.");
                return;
            }

            string timezoneOffset = botUser.TimezoneOffset > 0
                ? $"+{botUser.TimezoneOffset}"
                : $"{botUser.TimezoneOffset}";
            string description = match.Groups[1].Value;
            DateTime dateTime =
                DateTime.ParseExact($"{match.Groups[2].Value}UTC{timezoneOffset}", "dd/MM/yyyy HH:mmUTCz",
                    CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal);

            Recurring recurring = (Recurring)Enum.Parse(typeof(Recurring), match.Groups[3].Value, true);

            bool result = await Services.ExtremelyNaggyBotDB.Execute(
                new AddReminderQuery(new Reminder(chat.Id, description, dateTime.ToUniversalTime(), recurring)));

            if (result)
            {
                await Services.BotClient.SendTextMessageAsync(chat,
                    $"Hello {chat.FirstName}, I will remind you about {description} on {match.Groups[2].Value} {recurring.ToString().ToLower()}.");
            }
        }
    }
}