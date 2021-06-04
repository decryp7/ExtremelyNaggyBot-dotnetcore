using System;
using System.Reactive.Concurrency;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Threading;
using SimpleDatabase;

namespace ExtremelyNaggyBot
{
    public class Clock : DisposableObject, IClock
    {
        private readonly BehaviorSubject<DateTime> tickBehaviorSubject;

        public IObservable<DateTime> Tick => tickBehaviorSubject;

        public Clock()
        {
            Observable.Interval(TimeSpan.FromSeconds(1), Scheduler.Default)
                .Subscribe(l =>
                {
                    tickBehaviorSubject.OnNext(DateTime.Now.ToUniversalTime());
                })
                .DisposeWith(this);
        }
    }
}