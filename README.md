# Prometheus Periodic Commands

<div align="center">
    <a href="#example-use-case">Example Use-Case</a> •
    <a href="#features">Features</a> •
    <a href="#installation">Installation</a> •
    <a href="#configuration">Configuration</a> •
    <a href="#license">License</a> •
    <a href="#acknowledgments">Acknowledgments</a>
</div>

This tiny Rust program periodically runs commands that are specified in a config file
and parses the output of the commands using a RegEx. The output will be exposed as a
Prometheus metric.

### Example Use-Case

I use a storage box as a backup storage, but my backup system cannot tell me how much
storage is available. My solution is to use this tool to periodically connect to the storage
box using SSH to run the `du` command to find out how much storage is available.

With this tool, I can create AlertManager rules to warn me if the storage box comes close to being out of storage.

## Features

- 🧷 Multithreaded
- 🔒 Thread Safe
- 🎨 Colored Console Log
- 💾 YAML Config
- 🖥️ CLI Arguments

## Installation

This tool can be installed manually or using Docker.

### Manual Installation

1. Clone the Git repository and `cd` into the cloned directory

```sh
git clone https://github.com/JannesStroehlein/PrometheusPeriodicCommands.git
cd PrometheusPeriodicCommands
```

2. Build and install using cargo

```sh
cargo install
```

3. Run the tool

> [!NOTE]
> If the tool does not find a config file, it will exit.

```shell
./PrometheusPeriodicCommands
```

### Docker Installation

1. Pull the image

```shell
docker pull ghcr.io/jannesstroehlein/prometheusperiodiccommands:main
```

2. Create and run a container with the image

```shell
docker run -d \
  -p 8080:8080 \
  -v ${PWD}/config.yaml:/app/config.yaml:ro \
  --restart always \
  ghcr.io/jannesstroehlein/prometheusperiodiccommands:main \
  --host 127.0.0.1 --port 8080
```

> [!tip] Docker Compose
> Use the [pre-made docker compose file](docker-compose.yml) as your starting point.

**The Prometheus metrics are exposed under: `host:port/metrics`**

## Configuration

A YAML file is used to configure the tool. If no config file path is specified using the
`--config-file <path>` parameter, some paths are checked if they contain a `config.yaml` file.

<details>
<summary>OS specific config paths</summary>

| OS      | Paths                                                                                             |
|---------|---------------------------------------------------------------------------------------------------|
| Linux   | ~/.config/prometheus_periodic_commands/config.yaml, /etc/prometheus_periodic_commands/config.yaml |
| Windows | %LocalAppData%/prometheus_periodic_commands/config.yaml                                           |

</details>

### Config file format

```yaml
# The host to bind to
host: 0.0.0.0
# The port of the webserver
port: 8080

# A list of all commands to execute and parse
targets:
  # A shell command (Windows cmd, Linux: Bash)
  - command: echo 2
    # Specify the RegEx to parse the standard output of the command
    # The regex must include at least one named group (which needs to be specified below)
    # which is used to extract a numeric value from the command output
    regex: '(?<result>.*)'
    # The name of the group that contains the numeric result (the result can also be a float)
    regex_named_group: result
    # A list of exit codes that mark the execution as successful.
    # If the child process exits with an exit code not in this list,
    # the execution will be marked as faulted.
    success_exit_codes: [ 0 ]
    # Set the interval in which the command should be executed
    # You can use any format supported by https://docs.rs/duration-string/latest/duration_string/
    # Examples: 2h, 5h10m3s etc.
    run_every: 5s
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This tool uses these crates and other dependencies.

- actix-web - MIT - [GitHub](https://github.com/actix/actix-web)
- prometheus-client - MIT - [GitHub](https://github.com/prometheus/client_rust)
- serde - MIT - [GitHub](https://github.com/serde-rs/serde)
- serde_yaml - MIT - [GitHub](https://github.com/dtolnay/serde-yaml)
- duration-string - MIT [GitHub](https://github.com/RonniSkansing/duration-string)
- shellexpand - MIT - [GitLab](https://gitlab.com/ijackson/rust-shellexpand)
- simple_logger - MIT - [GitHub](https://github.com/borntyping/rust-simple_logger)
- tokio - MIT - [GitHub](https://github.com/tokio-rs/tokio)
- clap - MIT - [GitHub](https://github.com/clap-rs/clap)
