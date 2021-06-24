namespace ExtremelyNaggyBot.Sentry
{
    public static class SentryPerformanceMonitor
    {
        public static ISentryPerformanceMeasurement Measure(string name, string operation)
        {
            return new SentryPerformanceMeasurement(name, operation);
        }
    }
}