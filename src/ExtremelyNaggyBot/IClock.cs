using System;

namespace ExtremelyNaggyBot
{
    public interface IClock : IDisposable
    {
        IObservable<DateTime> Tick { get; }
    }
}