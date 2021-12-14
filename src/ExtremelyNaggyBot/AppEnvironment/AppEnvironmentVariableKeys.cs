using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ExtremelyNaggyBot.AppEnvironment
{
    internal class AppEnvironmentVariableKeys
    {
        public static AppEnvironmentVariableKey<string> TELEGRAM_BOT_TOKEN =
            new AppEnvironmentVariableKey<string>(nameof(TELEGRAM_BOT_TOKEN));

        public static AppEnvironmentVariableKey<long> ADMIN_CHATID =
            new AppEnvironmentVariableKey<long>(nameof(ADMIN_CHATID));

        public static AppEnvironmentVariableKey<string> SENTRY_DSN =
            new AppEnvironmentVariableKey<string>(nameof(SENTRY_DSN));
    }
}
