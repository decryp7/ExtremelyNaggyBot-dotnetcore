FROM mcr.microsoft.com/dotnet/sdk:5.0 AS build
WORKDIR /src

COPY *.csproj ./
RUN dotnet restore 

COPY /src/. .
RUN dotnet publish -c Release -o out

COPY bin/Release/net5.0/publish/ App/
WORKDIR /App
ENTRYPOINT dotnet ExtremelyNaggyBot.dll $TELEGRAM_BOT_TOKEN