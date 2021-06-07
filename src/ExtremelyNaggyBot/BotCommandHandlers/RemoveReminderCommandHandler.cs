using System;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.Query.Reminders;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class RemoveReminderCommandHandler : IBotCommandHandler
    {
        public string Command { get; } = "/removereminder";

        public string CommandDescription { get; } =
            "/removereminder {reminder_id}" +
            Environment.NewLine +
            "Remove reminder.";
        
        public async Task Handle(Chat chat, string commandArgs)
        {
            if (!long.TryParse(commandArgs, out long reminderId))
            {
                //send error message to user
                return;
            }

            bool reminderRemoved = await Services.ExtremelyNaggyBotDB.Execute(new RemoveReminderQuery(reminderId, chat.Id));

            if (reminderRemoved)
            {
               await Services.BotClient.SendTextMessageAsync(chat, $"Hello {chat.FirstName}, I have removed the reminder from the database.");
            }
        }
    }
}