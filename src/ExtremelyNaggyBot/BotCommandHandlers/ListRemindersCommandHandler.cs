using System;
using System.Collections.Generic;
using System.Text;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query.Reminders;
using ExtremelyNaggyBot.Database.Query.Users;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class ListRemindersCommandHandler : IBotCommandHandler
    {
        public string Command { get; } = "/listreminders";

        public string CommandDescription { get; } =
            "/listreminders" +
            Environment.NewLine +
            "List all your reminders.";
        
        public async Task Handle(Chat chat, string commandArgs)
        {
            IEnumerable<Reminder> reminders =
                await Services.ExtremelyNaggyBotDB.Execute(new GetRemindersQueryByUser(chat.Id));

            BotUser user = await Services.ExtremelyNaggyBotDB.Execute(new GetUserQuery(chat.Id));

            StringBuilder remindersStringBuilder = new StringBuilder();
            foreach (Reminder reminder in reminders)
            {
                DateTime userDateTime = TimeZoneInfo.ConvertTimeFromUtc(reminder.DateTime,
                    TimeZoneInfo.CreateCustomTimeZone("test", TimeSpan.FromHours(user.TimezoneOffset), null, null));
                remindersStringBuilder.AppendLine(
                    $"{reminder.Id}: {reminder.Description} at {userDateTime} {reminder.Recurring.ToString().ToLower()}");
            }

            string result = remindersStringBuilder.ToString();

            if (string.IsNullOrEmpty(result))
            {
                await Services.BotClient.SendTextMessageAsync(chat, $"Hello {chat.FirstName}, you do not have any reminders.");
                return;
            }

            await Services.BotClient.SendTextMessageAsync(chat,
                $"Hello {chat.FirstName}, your reminders are as follows,{Environment.NewLine}{result}");
        }
    }
}