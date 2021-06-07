using ExtremelyNaggyBot.BotCommandHandlers;
using SimpleDatabase;
using Telegram.Bot;

namespace ExtremelyNaggyBot
{
    public static class Services
    {
        public static IDatabase ExtremelyNaggyBotDB { get; set; }

        public static ITelegramBotClient BotClient { get; set; }

        public static IBotCommandHandlerService BotCommandHandlerService { get; set; }

        public static IClock Clock { get; set; }
    }
}