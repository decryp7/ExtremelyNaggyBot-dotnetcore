using GuardLibrary;

namespace ExtremelyNaggyBot.AppEnvironment
{
    internal class AppEnvironmentVariableKey<TSettingValue>
    {
        public string Name { get; }

        public AppEnvironmentVariableKey(string name)
        {
            Guard.Ensure(name, nameof(name)).IsNotNullOrEmpty();
            Name = name;
        }
    }
}
