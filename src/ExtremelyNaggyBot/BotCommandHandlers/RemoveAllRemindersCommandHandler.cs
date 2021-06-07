using System;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.Query.Reminders;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class RemoveAllRemindersCommandHandler : IBotCommandHandler
    {
        public string Command { get; } = "/removeallreminders";

        public string CommandDescription { get; } =
            "/removeallreminders" +
            Environment.NewLine +
            "Remove all your reminders.";
        
        public async Task Handle(Chat chat, string commandArgs)
        {
            int noOfRemindersRemoved =
                await Services.ExtremelyNaggyBotDB.Execute(new RemoveRemindersByUserQuery(chat.Id));

            if (noOfRemindersRemoved > 0)
            {
                await Services.BotClient.SendTextMessageAsync(chat, $"Hello {chat.FirstName}, I have removed the {noOfRemindersRemoved} reminders from the database.");
            }
        }
    }
}