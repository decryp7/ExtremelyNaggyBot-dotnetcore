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

        public IObservable<DateTime> Tick => tickBehaviorSubject;

        public Clock()
        {
            tickBehaviorSubject = new BehaviorSubject<DateTime>(DateTime.Now.ToUniversalTime())
                .DisposeWith(this);

            Observable.Interval(TimeSpan.FromSeconds(1), Scheduler.Default)
                .Subscribe(l =>
                {
                    tickBehaviorSubject.OnNext(DateTime.Now.ToUniversalTime());
                })
                .DisposeWith(this);
        }
    }
}