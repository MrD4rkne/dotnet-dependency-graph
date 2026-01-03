# Dotnet dependency graphs

[![Build and Test](https://github.com/MrD4rkne/dotnet-dependency-graph/actions/workflows/build.yaml/badge.svg)](https://github.com/MrD4rkne/dotnet-dependency-graph/actions/workflows/build.yaml)

Have you ever got lost in your app's dependency tree?

That's where this app comes to ease your pain.

It generates interactive trees of your dotnet project dependencies, allowing you to merge multiple project / solutions.

![Example of a dependency graph](/docs/example-graph.png)

## How to install it

### Git clone

1. Clone the repository:

    ```bash
    git clone https://github.com/MrD4rkne/dotnet-dependency-graph.git
    ```

2. Navigate to the project directory:

    ```bash
    cd dotnet-dependency-graph
    ```

3. Build the project using Cargo:

    ```bash
    cargo build --release
    ```

4. Next you can just run the app:

    ```bash
    cargo run --release
    ```

## Basic usage

### Open file

App is a tool for visualizing dependencies of .NET projects and solutions.
It parses `.dgspec.json` and `project.assets.json` files to build a dependency graph. You can either load a single file with `File -> Open file` menu option, which will load / replace the current graph.

![Dropdown at the left top corner- File -> Open file](/docs/select-file.png)

### Visualize edges

To visualize the depenendencies, you need to select the framework you are interested in. They are displayed at the top left corner of the window.

![Badges at the top left corner. Select the framework](/docs/select-framework.png)

### Merge

If you have an already loaded graph, you can merge another file into it using `File -> Merge file` menu option. This will try to combine the graphs, allowing you to visualize dependencies across multiple projects / solutions.

![Dropdown at the left top corner- File -> Merge file](/docs/merge-files.png)

### Navigate

You can navigate the graph using mouse interactions. Each node in the graph represents a dependency. Also, on the left side and bottom side of the window, you have lists of dependencies that you can also use to interacy with the graph.

- highlight nodes by hovering over,
- pan to the node by double clicking on it,
- select node by clicking on it,

Each of these interactions can be performed on the node itself, or on the dependency label in the left / bottom panel.

### Where to find the files

When working with .NET projects, during the `dotnet restore` process - when dependencies are being resolved - two important files are generated that contain information about the dependencies of your project:

- `project.assets.json`: This file is created in the `obj` directory of your project. It contains detailed information about the resolved dependencies, including their versions and how they relate to each other.
- `dgspec.json`: Also created in the `obj` directory. It contains the project's declared dependency specification — the inputs used by restore (declared PackageReference versions/ranges, project references, target frameworks, and restore metadata).

Example, if i have a project called `MyApp.csproj`, after running `dotnet restore`, you can find these files in the `obj` folder located in the same directory as your project file:

```MyApp/
├── MyApp.csproj
├── obj/
│   ├── project.assets.json
│   └── MyApp.csproj.nuget.dgspec.json
```

## Profiling

App uses `puffin` for profiling and `puffin_http` for exporting the data. To compile with profiling enabled, use the following command:

```bash
cargo build --release --features puffin-server
```

To visualize the profiling data, run the command that was logged in the console, for example:

```bash
puffin_viewer --url 127.0.0.1:8585
```

This will open a viewport where you can inspect the profiling data collected during the execution of the application.
Note that this requires the `puffin_viewer` tool to be installed on your system.
