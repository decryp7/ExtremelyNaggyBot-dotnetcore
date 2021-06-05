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

namespace ExtremelyNaggyBot
{
    class Program
    {
        private static ITelegramBotClient botClient;
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
                        9 when d.Minute == 0 && d.Second == 0 => "Good Morning!",
                        12 when d.Minute == 0 && d.Second == 0 => "Good Afternoon!",
                        15 when d.Minute == 0 && d.Second == 0 => "Nap Time!",
                        21 when d.Minute == 0 && d.Second == 0 => "Good Night!",
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
            switch (e.Message.Type)
            {
                case MessageType.WebsiteConnected:
                    await botClient.SendTextMessageAsync(
                        chatId: e.Message.Chat,
                        text: $"Hello {e.Message.From.FirstName}, Extremely Naggy Bot at your service."
                    );
                    break;
                default:
                    if (e.Message.Text != null)
                    {
                        Console.WriteLine($"Received a text message in chat {e.Message.Chat.Id}.");

                        await botClient.SendTextMessageAsync(
                            chatId: e.Message.Chat,
                            text: "You said:\n" + e.Message.Text
                        );
                    }
                    break;
            }
        }
    }
}
