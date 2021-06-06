﻿using System.Threading.Tasks;
using Telegram.Bot;
using Telegram.Bot.Types;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public interface IBotCommandHandler
    {
        ITelegramBotClient BotClient { set; }

        string Command { get; }

        string CommandDescription { get; }

        Task Handle(Chat chat, string commandArgs);
    }
}