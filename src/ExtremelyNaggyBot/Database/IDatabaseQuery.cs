namespace ExtremelyNaggyBot.Database
{
    public interface IDatabaseQuery<TDatabaseQuery, TDatabaseQueryResult>
        where TDatabaseQuery : IDatabaseQuery<TDatabaseQuery, TDatabaseQueryResult>
    {
        
    }
}