using System;
using System.Threading.Tasks;

namespace ExtremelyNaggyBot.Database
{
    public interface IDatabaseQueryHandler<TConnection, TDatabaseQuery, TDatabaseQueryResult>
        : IDatabaseQueryHandler
        where TDatabaseQuery : IDatabaseQuery<TDatabaseQuery, TDatabaseQueryResult>
    {
        Task<TDatabaseQueryResult> Handle(TConnection connection, TDatabaseQuery databaseQuery);
    }

    public interface IDatabaseQueryHandler
    {
    }
}