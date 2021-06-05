using System;
using System.Threading.Tasks;

namespace ExtremelyNaggyBot.Database
{
    public interface IDatabase : IDisposable
    {
        Task<TDatabaseQueryResult> Query<TDatabaseQuery, TDatabaseQueryResult>(
            IDatabaseQuery<TDatabaseQuery, TDatabaseQueryResult> databaseQuery)
            where TDatabaseQuery : IDatabaseQuery<TDatabaseQuery, TDatabaseQueryResult>;
    }

    public interface IDatabase<TConnection> : IDatabase
    {
        TConnection Connection { get; }
    }
}