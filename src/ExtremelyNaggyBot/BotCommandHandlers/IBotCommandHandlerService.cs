using System.Threading.Tasks;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public interface IBotCommandHandlerService
    {
        Task Handle(Chat chat, string command);

        string GetAvailableCommands();
    }
}