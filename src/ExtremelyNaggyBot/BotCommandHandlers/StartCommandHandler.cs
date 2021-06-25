using System.Threading.Tasks;
using Sentry;
using Telegram.Bot.Types;
using User = Telegram.Bot.Types.User;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class StartCommandHandler : IBotCommandHandler
    {
        public string Command { get; } = "/start";

        public string CommandDescription { get; }

        public async Task Handle(Chat chat, string commandArgs)
        {
            User bot = await Services.BotClient.GetMeAsync();
            await Services.BotClient.SendTextMessageAsync(chat, $"Hello {chat.FirstName}, {bot.FirstName} at your service.");
            SentrySdk.CaptureMessage($"I have received chat request from {chat.FirstName}.");
        }
    }
}