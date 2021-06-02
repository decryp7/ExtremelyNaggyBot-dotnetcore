FROM mcr.microsoft.com/dotnet/sdk:5.0 AS build
WORKDIR /src

COPY /src/ExtremelyNaggyBot/*.csproj ./
RUN dotnet restore 

COPY /src/ExtremelyNaggyBot/. ./
RUN dotnet publish -c Release -o out

RUN ls /src/out

COPY /src/out/. App/
WORKDIR /App
ENTRYPOINT dotnet ExtremelyNaggyBot.dll $TELEGRAM_BOT_TOKEN