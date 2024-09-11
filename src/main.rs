use config::{Config, File};
use regex::Regex;
use std::io::{self, BufReader, Read, Write};
use std::process::{exit, Command, Stdio};

fn main() {
    println!("正在开始同步……");
    // 检查配置文件是否存在
    if !std::path::Path::new("setting.toml").exists() {
        println!("哎我草！配置文件不存在！等等嗷，正在尝试新建一个 setting.toml 文件……");
        std::fs::write(
            "setting.toml",
            br#"
            [sync_file]
            local_path = "your local path"
            remote_path = "your remote path,default like onedrive:/the/path/you/want/to/sync"

            [lark]
            webhook = "https://open.feishu.cn/open-apis/bot/v2/hook/xxxxxx"
            "#,
        ).unwrap();
        println!("嘀嘀嘀！！配置文件已生成，请修改配置文件后重新运行程序！！");
        pause().unwrap();
        exit(1);
    } else {
        println!("配置文件存在，正在读取配置文件……");
    }
    // 读取配置文件
    let settings = Config::builder()
        .add_source(File::with_name("setting.toml").format(config::FileFormat::Toml))
        .build()
        .unwrap();
    let local_path = settings.get_string("sync_file.local_path").unwrap();
    let remote_path = settings.get_string("sync_file.remote_path").unwrap();
    let lark_webhook = settings.get_string("lark.webhook").unwrap();

    if local_path == "your local path" || remote_path == "your remote path,default like onedrive:/the/path/you/want/to/sync" || lark_webhook == "https://open.feishu.cn/open-apis/bot/v2/hook/xxxxxx" {
        eprintln!("布什哥们，你没改配置文件呢还！请修改配置文件后重新运行程序！");
        pause().unwrap();
        exit(2);
    } else if !std::path::Path::new("rclone.exe").exists() {
        eprintln!("我草！rclone.exe不存在！请你安一下rclone！安了的话请你把rclone放在本程序所在的目录下！！！");
        pause().unwrap();
        exit(3);
    }
    
    // 开始同步
    let mut child = Command::new("./rclone.exe")
        .arg("sync")
        .arg(&local_path)
        .arg(&remote_path)
        .arg("--progress")
        .arg("--color")
        .arg("ALWAYS")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");
    
    let stdout = child.stdout.as_mut().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);
    
    let mut buffer = Vec::new();
    for c in reader.bytes() {
        let c = c.unwrap();
        buffer.push(c);
        io::stdout().write_all(&[c]).unwrap();
    }

    let binding = String::from_utf8(buffer).unwrap();
    let lines: Vec<_> = binding.lines().collect();

    let status = child.wait().expect("Failed to wait on child");
    if status.success() {
        // 获取最后10行
        let num_lines = 10;
        let start = if lines.len() > num_lines { lines.len() - num_lines } else { 0 };
        let last_lines = &lines[start..];
        // 正则匹配
        let re_transferred = Regex::new(r"Transferred:\s+(\d+\.?\d* \w+) / (\d+\.?\d* \w+),").unwrap();
        let re_checks = Regex::new(r"Checks:\s+(\d+) / (\d+), (\d+)%").unwrap();
        let re_deleted = Regex::new(r"Deleted:\s+(\d+) \(files\), (\d+) \(dirs\), (\d+\.?\d* \w+) \(freed\)").unwrap();
        let re_transferred_simple = Regex::new(r"Transferred:\s+(\d+) / (\d+), (\d+)%").unwrap();
        let re_elapsed = Regex::new(r"Elapsed time:\s+(\d+m\d+\.?\d*s)").unwrap();
        // 匹配结果
        let mut transferred: Option<String> = None;
        let mut checks: Option<String> = None;
        let mut deleted: Option<String> = None;
        let mut transferred_simple: Option<String> = None;
        let mut elapsed_time: Option<String> = None;

        for line in last_lines {
            match (
                re_transferred.captures(line),
                re_checks.captures(line),
                re_deleted.captures(line),
                re_transferred_simple.captures(line),
                re_elapsed.captures(line),
            ) {
                (Some(caps), _, _, _, _) if transferred.is_none() => {
                    transferred = Some(format!("传输大小： {} / {},", &caps[1], &caps[2]));
                }
                (_, Some(caps), _, _, _) if checks.is_none() => {
                    checks = Some(format!("检查文件： {} / {}, {}%", &caps[1], &caps[2], &caps[3]));
                }
                (_, _, Some(caps), _, _) if deleted.is_none() => {
                    deleted = Some(format!("删除文件： {} (files), {} (dirs), {} (freed)", &caps[1], &caps[2], &caps[3]));
                }
                (_, _, _, Some(caps), _) if transferred_simple.is_none() => {
                    transferred_simple = Some(format!("共传输： {} / {}, {}%", &caps[1], &caps[2], &caps[3]));
                }
                (_, _, _, _, Some(caps)) if elapsed_time.is_none() => {
                    elapsed_time = Some(format!("总耗时： {}", &caps[1]));
                }
                _ => {}
            }
        }
        // 发送消息
        let _output = Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                r#"
                curl.exe -X POST -H "Content-Type: application/json" -d '{{
                    \"msg_type\":\"text\",
                    \"content\":{{
                        \"text\":\"{}\n{}\n{}\n{}\n{}\"
                    }}
                }}' {}
                "#,
                transferred.unwrap_or_else(|| "传输大小：无".to_string()),
                checks.unwrap_or_else(|| "检查：错误".to_string()),
                deleted.unwrap_or_else(|| "删除文件：无".to_string()),
                transferred_simple.unwrap_or_else(|| "共传输：未知".to_string()),
                elapsed_time.unwrap_or_else(|| "未知".to_string()),
                lark_webhook
            ))
            .output()
            .expect("Failed to execute command");
    } else {
        eprintln!("Command failed with status: {}", status);
    }
}

// 按任意键继续（）
fn pause() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\xE6\x8C\x89\xE4\xBB\xBB\xE6\x84\x8F\xE9\x94\xAE\xE7\xBB\xA7\xE7\xBB\xAD\xE2\x80\xA6\xE2\x80\xA6")?;
    stdout.flush()?;
    let mut stdin = io::stdin();
    let _ = stdin.read(&mut [0u8])?;
    Ok(())
}