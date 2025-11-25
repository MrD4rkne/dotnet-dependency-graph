# Dotnet dependency graphs

[![Build and Test](https://github.com/MrD4rkne/dotnet-dependency-graph/actions/workflows/build.yaml/badge.svg)](https://github.com/MrD4rkne/dotnet-dependency-graph/actions/workflows/build.yaml)

Have you ever got lost in your app's dependency tree?

That's where this app comes to ease your pain.

It generates interactive trees of your dotnet project dependencies, allowing you to merge multiple project / solutions.

## Features

- **Full Dependency Graph**: Parse `project.assets.json` to visualize both direct and transitive (indirect) dependencies
- **Legacy Support**: Also supports `.dgspec.json` files for backward compatibility (only shows direct dependencies)
- **Interactive Visualization**: Pan, zoom, and interact with the dependency graph
- **Framework Selection**: Switch between different target frameworks (e.g., net8.0, net9.0)
- **Package Filtering**: Show/hide specific packages to focus on what matters

## Usage

1. Build your .NET project/solution:

   ```bash
   dotnet restore
   dotnet build
   ```

2. Run the visualizer and load a `project.assets.json` file (recommended) or `.dgspec.json` file:
   - `project.assets.json` files are located in `obj/` directory of each project
   - They contain the **complete** resolved dependency graph including transitive dependencies

## File Formats

### project.assets.json (Recommended)

- Contains the **full resolved dependency graph** including all transitive dependencies
- Shows package-to-package dependencies (e.g., `Serilog.Sinks.Console` → `Serilog`)
- Located in `<project>/obj/project.assets.json`
- Generated after `dotnet restore`

### .dgspec.json (Legacy)

- Contains only **direct dependencies** from .csproj files
- Does not show transitive dependencies
- Located in `<project>/obj/<project>.csproj.nuget.dgspec.json`

## Example

For a project that references `Serilog.Sinks.Console`:

- **dgspec.json** shows: `MyProject` → `Serilog.Sinks.Console`
- **project.assets.json** shows: `MyProject` → `Serilog.Sinks.Console` → `Serilog` (complete graph)
