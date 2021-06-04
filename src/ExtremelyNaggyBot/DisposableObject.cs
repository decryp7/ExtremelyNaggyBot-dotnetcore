using System;
using System.Reactive.Disposables;

namespace ExtremelyNaggyBot
{
    public abstract class DisposableObject : IDisposable
    {
        private CompositeDisposable compositeDisposable;

        protected DisposableObject()
        {
            compositeDisposable = new CompositeDisposable();
        }

        #region IDisposable

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!disposing)
            {
                return;
            }

            if (compositeDisposable == null)
            {
                return;
            }

            compositeDisposable.Dispose();
            compositeDisposable = null;
        }

        #endregion

        public static implicit operator CompositeDisposable(DisposableObject disposable)
        {
            return disposable.compositeDisposable;
        }
    }
}