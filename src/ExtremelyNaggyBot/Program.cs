﻿using System;
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
using Telegram.Bot.Types.InlineQueryResults;
using Telegram.Bot.Types.ReplyMarkups;

namespace ExtremelyNaggyBot
{
    class Program
    {
        private static long adminChatId;

        static void Main(string[] args)
        {
            if (args.Length < 2)
            {
                Console.WriteLine("Missing parameters! Mandatory parameters are TELEGRAM_BOT_TOKEN ADMIN_CHATID in this order.");
                Environment.Exit(0);
            }

            Services.BotClient = new TelegramBotClient(args[0].Trim());
            adminChatId = long.Parse(args[1]);

            Services.BotCommandHandlerService = new BotCommandHandlerService(new IBotCommandHandler[]
            {
                new RegisterUserCommandHandler(),
                new RemoveUserCommandHandler()
            });

            Services.ExtremelyNaggyBotDB = new ExtremelyNaggyBotDB(Path.Combine("data", "extremelynaggybot.db"),
                new IDatabaseQueryHandler[]
                {
                    new SetupQueryHandler(),
                    new AddUserQueryHandler(),
                    new RemoveUserQueryHandler(),
                    new GetUserQueryHandler(),
                    new GetUsersQueryHandler()
                });

            if (Services.ExtremelyNaggyBotDB.Execute(new SetupQuery()).GetAwaiter().GetResult())
            {
                Console.WriteLine("Database is initialized!");
            }

            Services.Clock = new Clock();
            GreetingService greetingService = new GreetingService();

            try
            {
                var me = Services.BotClient.GetMeAsync().Result;
                Console.WriteLine(
                    $"Hello, World! I am user {me.Id} and my name is {me.FirstName}."
                );
            }
            catch (Exception ex)
            {
                Console.WriteLine($"There is an exception starting telegram bot client. {ex}");
                Environment.Exit(0);
            }

            Services.BotClient.OnMessage += BotClientOnOnMessage;
            Services.BotClient.OnCallbackQuery += BotClientOnOnCallbackQuery;
            Services.BotClient.StartReceiving();
            Services.BotClient.SendTextMessageAsync(new ChatId(adminChatId), "Extremely Naggy Bot is online!");

            Console.WriteLine("Press any key to exit");
            Console.ReadKey();

            Services.BotClient.StopReceiving();
            Services.Clock.Dispose();
        }

        private static async void BotClientOnOnCallbackQuery(object? sender, CallbackQueryEventArgs e)
        {
            await Services.BotClient.AnswerCallbackQueryAsync(e.CallbackQuery.Id, e.CallbackQuery.Data);
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
                    await Services.BotCommandHandlerService.Handle(e.Message.Chat,
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
                await Services.BotClient.SendTextMessageAsync(
                    chatId: e.Message.Chat,
                    text: $"Internal Error has occurred."
                );
            }
        }
    }
}
