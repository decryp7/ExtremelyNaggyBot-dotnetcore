using System;
using System.Globalization;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query.Reminders;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class RemindMeCommandHandler : IBotCommandHandler
    {
        private readonly Regex remindmeRegex =
            new Regex(@"(.*) (\d{1,2}\/\d{1,2}\/\d{4} \d{1,2}:\d{1,2}) (once|daily|weekly|monthly)", RegexOptions.Compiled | RegexOptions.IgnoreCase);

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

            string description = match.Groups[1].Value;
            DateTime dateTime =
                DateTime.ParseExact(match.Groups[2].Value, "dd/MM/yyyy HH:mm", CultureInfo.InvariantCulture);

            Recurring recurring = (Recurring)Enum.Parse(typeof(Recurring), match.Groups[3].Value, true);

            bool result = await Services.ExtremelyNaggyBotDB.Execute(
                new AddReminderQuery(new Reminder(chat.Id, description, dateTime.ToUniversalTime(), recurring)));

            if (result)
            {
                await Services.BotClient.SendTextMessageAsync(chat,
                    $"Hello {chat.FirstName}, I will remind you about {description} on {dateTime:dd/MM/yyyy HH:mm} {recurring.ToString().ToLower()}.");
            }
        }
    }
}