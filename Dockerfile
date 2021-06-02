FROM mcr.microsoft.com/dotnet/sdk:5.0 as build-env
WORKDIR /src

# Copy csproj and restore as distinct layers
COPY /src/ExtremelyNaggyBot/*.csproj ./
RUN dotnet restore 

# Copy everything else and build
COPY /src/ExtremelyNaggyBot ./
RUN dotnet publish -c Release -o out

# Build runtime image
FROM mcr.microsoft.com/dotnet/aspnet:5.0
WORKDIR /App
COPY --from=build-env /src/out .
ENTRYPOINT dotnet ExtremelyNaggyBot.dll $TELEGRAM_BOT_TOKEN