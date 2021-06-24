using System;
using System.Collections.Generic;
using System.Text;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Sentry;
using GuardLibrary;
using Telegram.Bot;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class BotCommandHandlerService : IBotCommandHandlerService
    {
        private readonly IDictionary<string, IBotCommandHandler> botCommandHandlers =
            new Dictionary<string, IBotCommandHandler>(StringComparer.OrdinalIgnoreCase);

        public BotCommandHandlerService(IBotCommandHandler[] handlers)
        {
            foreach (IBotCommandHandler botCommandHandler in handlers)
            {
                botCommandHandlers[botCommandHandler.Command] = botCommandHandler;
            }
        }

        public async Task Handle(Chat chat, string command)
        {
            Guard.Ensure(command, nameof(command)).IsNotNullOrEmpty();

            int indexOfFirstSpace = command.IndexOf(' ');

            string cmd = command;
            string cmdArgs = string.Empty;
            if (indexOfFirstSpace > -1)
            {
                cmd = command.Substring(0, indexOfFirstSpace);
                cmdArgs = command.Substring(indexOfFirstSpace);
            }

            using (SentryPerformanceMonitor.Measure(cmd, command))
            {
                if (!botCommandHandlers.TryGetValue(cmd, out IBotCommandHandler botCommandHandler))
                {
                    await Services.BotClient.SendTextMessageAsync(chat,
                        $"Sorry {chat.FirstName}, I am unable to handle {command}.");
                    await Services.BotClient.SendTextMessageAsync(chat, $"{this.GetAvailableCommands()}");
                    return;
                }

                await botCommandHandler.Handle(chat, cmdArgs);
            }
        }

        public string GetAvailableCommands()
        {
            StringBuilder stringBuilder = new StringBuilder("Available Commands:" + Environment.NewLine);

            int commandIndex = 0;
            foreach (KeyValuePair<string, IBotCommandHandler> botCommandHandler in botCommandHandlers)
            {
                if (string.IsNullOrEmpty(botCommandHandler.Value.CommandDescription))
                {
                    continue;
                }

                stringBuilder.AppendLine($"{++commandIndex}. {botCommandHandler.Value.CommandDescription}");
                stringBuilder.AppendLine("");
            }

            return stringBuilder.ToString();
        }
    }
}