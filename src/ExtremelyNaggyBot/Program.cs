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
using System.Net.Http.Headers;
using System.Text.RegularExpressions;
using ExtremelyNaggyBot.BotCommandHandlers;
using ExtremelyNaggyBot.Database;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query;
using ExtremelyNaggyBot.Database.Query.Reminders;
using ExtremelyNaggyBot.Database.Query.Users;
using Newtonsoft.Json;
using Sentry;
using SimpleDatabase;
using Telegram.Bot.Types.InlineQueryResults;
using Telegram.Bot.Types.ReplyMarkups;
using JsonSerializer = System.Text.Json.JsonSerializer;

namespace ExtremelyNaggyBot
{
    class Program
    {
        private static long adminChatId;

        static void Main(string[] args)
        {
            using (SentrySdk.Init(o =>
            {
                o.Dsn = "https://6dc01bc529194efe95cb57e14b9f5fa0@sentry.decryptology.net/2";
                // When configuring for the first time, to see what the SDK is doing:
                //o.Debug = true;
                // Set traces_sample_rate to 1.0 to capture 100% of transactions for performance monitoring.
                // We recommend adjusting this value in production.
                o.TracesSampleRate = 1.0;
            }))
            {
                // App code goes here. Dispose the SDK before exiting to flush events.

                if (args.Length < 2)
                {
                    Console.WriteLine(
                        "Missing parameters! Mandatory parameters are TELEGRAM_BOT_TOKEN ADMIN_CHATID in this order.");
                    Environment.Exit(0);
                }

                Services.BotClient = new TelegramBotClient(args[0].Trim());
                adminChatId = long.Parse(args[1]);

                Services.BotCommandHandlerService = new BotCommandHandlerService(new IBotCommandHandler[]
                {
                    new StartCommandHandler(),
                    new AboutCommandHandler(),
                    //users
                    new RegisterUserCommandHandler(),
                    new UnregisterUserCommandHandler(),
                    //reminders
                    new RemindMeCommandHandler(),
                    new ListRemindersCommandHandler(),
                    new RemoveReminderCommandHandler(),
                    new RemoveAllRemindersCommandHandler()
                });

                Services.ExtremelyNaggyBotDB = new ExtremelyNaggyBotDB(Path.Combine("data", "extremelynaggybot.db"),
                    new IDatabaseQueryHandler[]
                    {
                        new SetupQueryHandler(),
                        new CleanupQueryHandler(),
                        new VacuumQueryHandler(),
                        //users
                        new AddUserQueryHandler(),
                        new RemoveUserQueryHandler(),
                        new GetUserQueryHandler(),
                        new GetUsersQueryHandler(),
                        //reminders
                        new AddReminderQueryHandler(),
                        new RemoveReminderHandler(),
                        new GetReminderQueryHandler(),
                        new GetRemindersQueryByUserHandler(),
                        new GetRemindersQueryHandler(),
                        new RemoveRemindersByUserQueryHandler(),
                        new AcknowledgeReminderQueryHandler(),
                        //nagging
                        new AddNaggingQueryHandler(),
                        new GetNaggingsQueryHandler(),
                        new UpdateNaggingDatetimeQueryHandler()
                    });

                if (Services.ExtremelyNaggyBotDB.Execute(new SetupQuery()).GetAwaiter().GetResult())
                {
                    Console.WriteLine("Database is initialized!");
                }

                if (Services.ExtremelyNaggyBotDB.Execute(new CleanupQuery()).GetAwaiter().GetResult())
                {
                    Console.WriteLine("Database is cleaned up!");
                }

                Services.Clock = new Clock();
                ReminderService reminderService = new ReminderService();
                NaggingService naggingService = new NaggingService();
                DatabaseVacuumService databaseVacuumService = new DatabaseVacuumService();

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
        }

        private static async void BotClientOnOnCallbackQuery(object? sender, CallbackQueryEventArgs e)
        {
            ReminderAcknowledgement reminderAcknowledgement = JsonSerializer.Deserialize<ReminderAcknowledgement>(e.CallbackQuery.Data);

            if (reminderAcknowledgement != null)
            {
                await Services.ExtremelyNaggyBotDB.Execute(new AcknowledgeReminderQuery(reminderAcknowledgement));
            }

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
                SentrySdk.CaptureMessage(msg);
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
