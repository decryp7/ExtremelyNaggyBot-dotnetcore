using GuardLibrary;
using Sentry;

namespace ExtremelyNaggyBot.Sentry
{
    public class SentryPerformanceMeasurement : ISentryPerformanceMeasurement
    {
        private readonly ITransaction transaction;

        public SentryPerformanceMeasurement(string name, string operation)
        {
            Guard.Ensure(name, nameof(name)).IsNotNullOrEmpty();
            Guard.Ensure(operation, nameof(operation)).IsNotNullOrEmpty();

            transaction = SentrySdk.StartTransaction(name, operation);
        }

        public void Dispose()
        {
            transaction?.Finish();
        }
    }
}