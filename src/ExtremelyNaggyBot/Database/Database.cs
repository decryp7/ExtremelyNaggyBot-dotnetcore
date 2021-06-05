using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Threading.Tasks;

namespace ExtremelyNaggyBot.Database
{
    public abstract class Database<TConnection> : DisposableObject, IDatabase<TConnection>
    {
        private readonly IDictionary<Tuple<Type, Type, Type>, IDatabaseQueryHandler> databaseQueryHandlers =
            new Dictionary<Tuple<Type, Type, Type>, IDatabaseQueryHandler>();

        protected Database(IEnumerable<IDatabaseQueryHandler> dbQueryhandlers)
        {
            foreach (IDatabaseQueryHandler databaseQueryHandler in dbQueryhandlers)
            {
                Tuple<Type, Type, Type> queryType = FindQueryHandlerType(databaseQueryHandler.GetType());
                if (queryType != null)
                {
                    databaseQueryHandlers[queryType] = databaseQueryHandler;
                }
            }
        }

        private Tuple<Type, Type, Type> FindQueryHandlerType(Type baseType)
        {
            Type queryType = baseType.GetInterfaces().FirstOrDefault(x =>
                x.IsGenericType &&
                x.GetGenericTypeDefinition() == typeof(IDatabaseQueryHandler<,,>));

            if (queryType != null)
            {
                return new Tuple<Type, Type, Type>(queryType.GenericTypeArguments[0], queryType.GenericTypeArguments[1],
                    queryType.GenericTypeArguments[2]);
            }

            return FindQueryHandlerType(baseType.BaseType);
        }

        public async Task<TDatabaseQueryResult> Query<TDatabaseQuery, TDatabaseQueryResult>(IDatabaseQuery<TDatabaseQuery, TDatabaseQueryResult> databaseQuery)
            where TDatabaseQuery : IDatabaseQuery<TDatabaseQuery, TDatabaseQueryResult>
        {
            if (databaseQueryHandlers.TryGetValue(new Tuple<Type, Type, Type>(typeof(TConnection), typeof(TDatabaseQuery), typeof(TDatabaseQueryResult)),
                out IDatabaseQueryHandler databaseQueryHandler))
            {
                return await ((IDatabaseQueryHandler<TConnection, TDatabaseQuery, TDatabaseQueryResult>)databaseQueryHandler).Handle(Connection,
                    (TDatabaseQuery)databaseQuery);
            }

            return default(TDatabaseQueryResult);
        }

        public abstract TConnection Connection { get; }
    }
}