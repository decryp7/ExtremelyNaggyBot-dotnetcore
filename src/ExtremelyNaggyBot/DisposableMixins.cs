using System;
using System.Reactive.Disposables;

namespace ExtremelyNaggyBot
{
    public static class DisposableMixins
    {
        /// <summary>
        /// Ensures the provided disposable is disposed with the specified <see cref="T:System.Reactive.Disposables.CompositeDisposable" />.
        /// </summary>
        /// <typeparam name="T">The type of the disposable.</typeparam>
        /// <param name="this">The disposable.</param>
        /// <param name="compositeDisposable">
        /// The <see cref="T:System.Reactive.Disposables.CompositeDisposable" /> to which <paramref name="this" /> will be added.
        /// </param>
        /// <returns>The disposable.</returns>
        public static T DisposeWith<T>(this T @this, CompositeDisposable compositeDisposable) where T : IDisposable
        {
            compositeDisposable.Add((IDisposable)@this);
            return @this;
        }
    }
}