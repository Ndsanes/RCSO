use config::{Config, File};
use regex::Regex;
use std::io;
use std::io::Read;
use std::io::{BufReader, Stdout, Write};
use std::process::{Command, Stdio};

fn main() {
    // 读取配置文件
    let settings = Config::builder()
        .add_source(File::with_name(r"setting.toml").format(config::FileFormat::Toml))
        .build()
        .unwrap();
    let local_path = settings.get_string("sync_file.local_path").unwrap();
    let remote_path = settings.get_string("sync_file.remote_path").unwrap();
    let lark_webhook = settings.get_string("lark.webhook").unwrap();

    // 启动子进程
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
        Stdout::write(&mut io::stdout(), &[c]).unwrap();
    }

    let binding = String::from_utf8(buffer).unwrap();
    let lines: Vec<_> = binding.lines().collect();

    let status = child.wait().expect("Failed to wait on child");
    if status.success() {
        let num_lines = 10; // 输出最后10行
        let start = if lines.len() > num_lines {
            lines.len() - num_lines
        } else {
            0
        };
        let mut last_lines = lines[start..].to_vec();
        last_lines.reverse(); // 颠倒顺序

        // 定义正则表达式，处理特殊字符
        let re_transferred = Regex::new(r"Transferred:\s+(\d+\.?\d* \w+) / (\d+\.?\d* \w+),").unwrap();
        let re_checks = Regex::new(r"Checks:\s+(\d+) / (\d+), (\d+)%").unwrap();
        let re_deleted = Regex::new(r"Deleted:\s+(\d+) \(files\), (\d+) \(dirs\), (\d+\.?\d* \w+) \(freed\)").unwrap();
        let re_transferred_simple = Regex::new(r"Transferred:\s+(\d+) / (\d+), (\d+)%").unwrap();
        let re_elapsed = Regex::new(r"Elapsed time:\s+(\d+m\d+\.?\d*s)").unwrap();

        // 设置默认值
        let mut transferred = "未知".to_string();
        let mut checks = "未知".to_string();
        let mut deleted = "未知".to_string();
        let mut transferred_simple = "未知".to_string();
        let mut elapsed_time = "未知".to_string();

        // 匹配数据
        for line in &last_lines {
            if transferred == "未知" {
                if let Some(caps) = re_transferred.captures(line) {
                    transferred = format!("传输文件： {} / {},", &caps[1], &caps[2]);
                }
            }
            if checks == "未知" {
                if let Some(caps) = re_checks.captures(line) {
                    checks = format!("检查文件： {} / {}, {}%", &caps[1], &caps[2], &caps[3]);
                }
            }
            if deleted == "未知" {
                if let Some(caps) = re_deleted.captures(line) {
                    deleted = format!("删除文件： {} (files), {} (dirs), {} (freed)", &caps[1], &caps[2], &caps[3]);
                }
            }
            if transferred_simple == "未知" {
                if let Some(caps) = re_transferred_simple.captures(line) {
                    transferred_simple = format!("共传输： {} / {}, {}%", &caps[1], &caps[2], &caps[3]);
                }
            }
            if elapsed_time == "未知" {
                if let Some(caps) = re_elapsed.captures(line) {
                    elapsed_time = format!("总耗时： {}", &caps[1]);
                }
            }
        }

        // 打印结果
        println!("=====================");
        println!("{}", transferred);
        println!("{}", checks);
        println!("{}", deleted);
        println!("{}", transferred_simple);
        println!("{}", elapsed_time);
        println!("=====================");

        // 开启powershell脚本子进程，将结果以参数的格式发送到脚本中
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
                transferred, checks, deleted, transferred_simple, elapsed_time, lark_webhook
            ))
            .output()
            .expect("Failed to execute command");
    } else {
        eprintln!("Command failed with status: {}", status);
    }
}