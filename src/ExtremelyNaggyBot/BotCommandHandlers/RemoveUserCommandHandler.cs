using System;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database;
using ExtremelyNaggyBot.Database.Query;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class RemoveUserCommandHandler : IBotCommandHandler
    {
        public string Command { get; } = "/remove";

        public string CommandDescription { get; } =
            "/remove" +
            Environment.NewLine +
            "Remove yourself from bot's database.";

        public async Task Handle(Chat chat, string commandArgs)
        {
            bool userRemoved = await Services.ExtremelyNaggyBotDB.Execute(new RemoveUserQuery(chat.Id));

            if (!userRemoved)
            {
                //Send error message
                return;
            }

            await Services.BotClient.SendTextMessageAsync(chat, $"Hello {chat.FirstName}, I have removed you from my database.");
        }
    }
}