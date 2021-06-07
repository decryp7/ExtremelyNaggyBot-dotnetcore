using System;
using System.Threading.Tasks;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class AboutCommandHandler : IBotCommandHandler
    {
        public string Command { get; } = "/about";

        public string CommandDescription { get; } =
            "/about" +
            Environment.NewLine +
            "Display information about this bot.";
        
        public async Task Handle(Chat chat, string commandArgs)
        {
            await Services.BotClient.SendTextMessageAsync(chat, 
                @$"Hello {chat.FirstName}, I am a extremely naggy reminder bot.
Contact for bug reports: @decryp7
Source code: https://dev.decryptology.net/decryp7/ExtremelyNaggyBot-dotnetcore");
        }
    }
}