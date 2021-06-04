using System;
using System.Reactive.Concurrency;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Threading;

namespace ExtremelyNaggyBot
{
    public class Clock : DisposableObject, IClock
    {
        private readonly BehaviorSubject<DateTime> tickBehaviorSubject;

        public IObservable<DateTime> Tick => tickBehaviorSubject.DistinctUntilChanged();

        public Clock()
        {
            tickBehaviorSubject = new BehaviorSubject<DateTime>(DateTime.Now.ToUniversalTime())
                .DisposeWith(this);

            Observable.Interval(TimeSpan.FromMinutes(1), Scheduler.Default)
                .Subscribe(l =>
                {
                    DateTime utcDateTime = DateTime.Now.ToUniversalTime();
                    tickBehaviorSubject.OnNext(new DateTime(utcDateTime.Year, utcDateTime.Month, utcDateTime.Day, utcDateTime.Hour,
                        utcDateTime.Minute, 0, DateTimeKind.Utc));
                })
                .DisposeWith(this);
        }
    }
}