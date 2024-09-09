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

