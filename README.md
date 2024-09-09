以下是一个简单的 `README.md` 文件示例，描述了你的 Rust 项目及其功能。


# Rclone Backup Script

This Rust project is a script to synchronize files using `rclone` and send a summary of the operation to a Lark webhook.

## Prerequisites

- Rust
- `rclone`
- A Lark webhook URL
- A configuration file in TOML format

## Configuration

Create a `setting.toml` file with the following structure:

```toml
[sync_file]
local_path = "path/to/local/directory"
remote_path = "path/to/remote/directory"

[lark]
webhook = "https://open.feishu.cn/open-apis/bot/v2/hook/****"
```

## Usage

1. Clone the repository.
2. Ensure you have `rclone` installed and accessible in your PATH.
3. Create the `setting.toml` configuration file as described above.
4. Build and run the project using Cargo:

```sh
cargo build
cargo run
```

## How It Works

1. The script reads the configuration file to get the local and remote paths and the Lark webhook URL.
2. It starts an `rclone` process to synchronize the files.
3. It captures the output of the `rclone` process and extracts relevant information using regular expressions.
4. It sends a summary of the operation to the specified Lark webhook using a PowerShell command.

## Dependencies

- `config`: For reading the configuration file.
- `regex`: For parsing the `rclone` output.
- `reqwest`: For sending HTTP requests.
- `serde_json`: For constructing JSON payloads.

## License

This project is licensed under the MIT License.
