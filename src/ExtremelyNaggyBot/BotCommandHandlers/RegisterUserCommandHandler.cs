using System;
using System.Threading.Tasks;
using ExtremelyNaggyBot.Database.DataModel;
using ExtremelyNaggyBot.Database.Query;
using ExtremelyNaggyBot.Database.Query.Users;
using Telegram.Bot;
using Telegram.Bot.Types;
using User = ExtremelyNaggyBot.Database.DataModel.BotUser;

namespace ExtremelyNaggyBot.BotCommandHandlers
{
    public class RegisterUserCommandHandler : IBotCommandHandler
    {
        public string Command { get; } = "/register";

        public string CommandDescription { get; } =
            "/register {timezone_offset}" +
            Environment.NewLine +
            "Register yourself with timezone offset";
        
        public async Task Handle(Chat chat, string commandArgs)
        {
            int timezoneOffset = int.Parse(commandArgs);

            bool userAdded =
                await Services.ExtremelyNaggyBotDB.Execute(new AddUserQuery(new User(chat.Id, chat.FirstName,
                    chat.LastName, timezoneOffset)));

            if (!userAdded)
            {
                //Send error message to user
                return;
            }

            await Services.BotClient.SendTextMessageAsync(chat, $"Hello {chat.FirstName}, I have registered you with timezone offset: {timezoneOffset}.");
        }
    }
}