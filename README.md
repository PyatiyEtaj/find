# rfind

interactive mode
```
$user> rfind
> SomeApi
1) ./SomeApi/appsettings.Development.json
2) ./SomeApi/appsettings.json
3) ./SomeApi/Attributes/AddDependencies.cs
4) ./SomeApi/Attributes/ExportAttribute.cs
5) ./SomeApi/Attributes/FilteringAttribute.cs
6) ./SomeApi/Attributes/UseDateTimeFilterAttribute.cs
7) ./SomeApi/Attributes/UseDependenciesResolverAttribute.cs
8) ./SomeApi/Attributes/UseMessageTypeFilterAttribute.cs
9) ./SomeApi/Attributes/UseProjectToClientAttribute.cs
10) ./SomeApi/Attributes/UseRequestTypeAttribute.cs
... some more

> type again
```

simple search
```
$user> rfind my_pattern
1) ./SomeApi/appsettings.Development.json
2) ./SomeApi/appsettings.json
3) ./SomeApi/Attributes/AddDependencies.cs
4) ./SomeApi/Attributes/ExportAttribute.cs
5) ./SomeApi/Attributes/FilteringAttribute.cs
6) ./SomeApi/Attributes/UseDateTimeFilterAttribute.cs
7) ./SomeApi/Attributes/UseDependenciesResolverAttribute.cs
8) ./SomeApi/Attributes/UseMessageTypeFilterAttribute.cs
9) ./SomeApi/Attributes/UseProjectToClientAttribute.cs
10) ./SomeApi/Attributes/UseRequestTypeAttribute.cs
...
199) ./SomeApi/Attributes/SomeFile.cs
$user>
```