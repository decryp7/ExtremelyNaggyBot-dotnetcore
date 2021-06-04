using System;
using System.Reactive.Disposables;
using System.Reactive.Subjects;
using System.Threading;
using SimpleDatabase;

namespace ExtremelyNaggyBot
{
    public class Clock : DisposableObject, IClock
    {
        private readonly Timer timer;
        private readonly BehaviorSubject<DateTime> tickBehaviorSubject;

        public IObservable<DateTime> Tick => tickBehaviorSubject;

        public Clock()
        {
            timer = new Timer(Callback, null, 0, 1000)
                .DisposeWith(this);
            tickBehaviorSubject = new BehaviorSubject<DateTime>(DateTime.Now.ToUniversalTime())
                .DisposeWith(this);
        }

        private void Callback(object? state)
        {
            tickBehaviorSubject.OnNext(DateTime.Now.ToUniversalTime());
        }
    }
}