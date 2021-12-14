using System;

namespace ExtremelyNaggyBot.AppEnvironment
{
    internal class AppEnvironmentVariables
    {
        public TSettingValue Read<TSettingValue>(AppEnvironmentVariableKey<TSettingValue> appEnvironmentVariableKey)
        {
            string? environmentVariable = Environment.GetEnvironmentVariable(appEnvironmentVariableKey.Name);
            if (environmentVariable != null)
            {
                return (TSettingValue)Convert.ChangeType(environmentVariable, typeof(TSettingValue));
            }

            return default;
        }
    }
}