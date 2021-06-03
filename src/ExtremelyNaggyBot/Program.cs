using System;
using System.Threading;
using Telegram.Bot;
using Telegram.Bot.Args;
using Telegram.Bot.Types;
using Telegram.Bot.Types.Enums;

namespace ExtremelyNaggyBot
{
    class Program
    {
        private static ITelegramBotClient botClient;

        static void Main(string[] args)
        {
            if (args.Length < 1)
            {
                Console.WriteLine("Missing telegram token parameter!");
                Environment.Exit(0);
            }

            botClient = new TelegramBotClient(args[0].Trim());

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

            botClient.OnMessage += BotClientOnOnMessage;
            botClient.StartReceiving();

            Console.WriteLine("Press any key to exit");
            Console.ReadKey();

            botClient.StopReceiving();
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
