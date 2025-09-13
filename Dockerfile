FROM mcr.microsoft.com/dotnet/sdk:10.0 as build-env
WORKDIR /src

# Copy csproj and restore as distinct layers
COPY /src/ExtremelyNaggyBot/*.csproj ./
RUN dotnet nuget add source https://repository.decryptology.net/repository/Nuget/ -n decryptology.net
RUN dotnet restore 

# Copy everything else and build
COPY /src/ExtremelyNaggyBot ./
RUN dotnet publish -c Release -o out

# Build runtime image
FROM mcr.microsoft.com/dotnet/aspnet:10.0
WORKDIR /app
COPY --from=build-env /src/out .
ENTRYPOINT dotnet ExtremelyNaggyBot.dll