using System;
using System.Collections.ObjectModel;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Reflection;
using System.Threading;
using Telegram.Bot;
using Telegram.Bot.Args;
using Telegram.Bot.Types;
using Telegram.Bot.Types.Enums;
using System.IO;
using ExtremelyNaggyBot.BotCommandHandlers;
using ExtremelyNaggyBot.Database;
using ExtremelyNaggyBot.Database.Query;
using SimpleDatabase;

namespace ExtremelyNaggyBot
{
    class Program
    {
        private static ITelegramBotClient botClient;
        private static IBotCommandHandlerService botCommandHandlerService;
        private static long adminChatId;

        static void Main(string[] args)
        {
            if (args.Length < 2)
            {
                Console.WriteLine("Missing parameters! Mandatory parameters are TELEGRAM_BOT_TOKEN ADMIN_CHATID in this order.");
                Environment.Exit(0);
            }

            botClient = new TelegramBotClient(args[0].Trim());
            adminChatId = long.Parse(args[1]);

            botCommandHandlerService = new BotCommandHandlerService(botClient);

            try
            {
                var me = botClient.GetMeAsync().Result;
                Console.WriteLine(
                    $"Hello, World! I am user {me.Id} and my name is {me.FirstName}."
                );
            }
            catch (Exception ex)
            {
                Console.WriteLine($"There is an exception starting telegram bot client. {ex}");
                Environment.Exit(0);
            }

            IDatabase database = new ExtremelyNaggyBotDB(Path.Combine("data", "extremelynaggybot.db"),
                new IDatabaseQueryHandler[]
                {
                    new SetupQueryHandler()
                });

            if (database.Execute(new SetupQuery()).GetAwaiter().GetResult())
            {
                Console.WriteLine("Database is initialized!");
            }

            IClock clock = new Clock();
            IDisposable subscription = clock.Tick
                .ObserveOn(Scheduler.Default)
                .Subscribe(async dateTime =>
                {
                    string message = string.Empty;

                    DateTime d = TimeZoneInfo.ConvertTimeFromUtc(dateTime,
                        TimeZoneInfo.CreateCustomTimeZone("test", TimeSpan.FromHours(8), null, null));
                    message = d.Hour switch
                    {
                        9 when d.Minute == 0 => "Good Morning!",
                        12 when d.Minute == 0 => "Good Afternoon!",
                        15 when d.Minute == 0 => "Nap Time!",
                        21 when d.Minute == 0 => "Good Night!",
                        _ => message
                    };

                    if (string.IsNullOrEmpty(message))
                    {
                        return;
                    }

                    await botClient.SendTextMessageAsync(new ChatId(adminChatId), message);
                });

            botClient.OnMessage += BotClientOnOnMessage;
            botClient.StartReceiving();
            botClient.SendTextMessageAsync(new ChatId(adminChatId), "Extremely Naggy Bot is online!");

            Console.WriteLine("Press any key to exit");
            Console.ReadKey();

            botClient.StopReceiving();
            subscription.Dispose();
            clock.Dispose();
        }

        private static async void BotClientOnOnMessage(object sender, MessageEventArgs e)
        {
            try
            {
                if (e.Message.Entities != null &&
                    e.Message.Entities.Length > 0 &&
                    e.Message.Entities[0].Type == MessageEntityType.BotCommand)
                {
                    string command = e.Message.Text.Substring(e.Message.Entities[0].Offset);
                    await botCommandHandlerService.Handle(e.Message.Chat,
                        e.Message.Text.Substring(e.Message.Entities[0].Offset));

                    return;
                }

                if (e.Message.Text == null)
                {
                    return;
                }

                string msg = $"Received a text message from {e.Message.From.FirstName}. Message: {e.Message.Text}";
                Console.WriteLine(msg);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Internal Error has occurred. {ex}");
                await botClient.SendTextMessageAsync(
                    chatId: e.Message.Chat,
                    text: $"Internal Error has occurred."
                );
            }
        }
    }
}
