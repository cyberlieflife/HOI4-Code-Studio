use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
pub struct TerminalState {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
}

struct TerminalSession {
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionInfo {
    session_id: String,
    shell: String,
    cwd: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalOutputEvent {
    session_id: String,
    stream: String,
    text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalExitEvent {
    session_id: String,
    exit_code: Option<i32>,
}

fn normalize_shell(shell: &str) -> Result<&'static str, String> {
    match shell.trim().to_lowercase().as_str() {
        "cmd" => Ok("cmd"),
        "powershell" => Ok("powershell"),
        _ => Err("仅支持 cmd 和 powershell 终端".to_string()),
    }
}

fn create_shell_command(shell: &str) -> (String, Vec<String>) {
    match shell {
        "cmd" => (
            "cmd.exe".to_string(),
            vec!["/Q".to_string(), "/K".to_string(), "chcp 65001>nul".to_string()],
        ),
        "powershell" => (
            "powershell.exe".to_string(),
            vec![
                "-NoLogo".to_string(),
                "-NoExit".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                "[Console]::InputEncoding=[System.Text.UTF8Encoding]::new();[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new();$OutputEncoding=[System.Text.UTF8Encoding]::new()".to_string(),
            ],
        ),
        _ => unreachable!(),
    }
}

fn spawn_terminal_reader<R: Read + Send + 'static>(
    mut reader: R,
    app: AppHandle,
    session_id: String,
    stream: &'static str,
) {
    thread::spawn(move || {
        let mut buffer = [0_u8; 2048];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let text = String::from_utf8_lossy(&buffer[..size]).to_string();
                    let _ = app.emit(
                        "terminal-output",
                        TerminalOutputEvent {
                            session_id: session_id.clone(),
                            stream: stream.to_string(),
                            text,
                        },
                    );
                }
                Err(error) => {
                    let _ = app.emit(
                        "terminal-output",
                        TerminalOutputEvent {
                            session_id: session_id.clone(),
                            stream: "system".to_string(),
                            text: format!("\n[系统] 终端输出读取失败: {}\n", error),
                        },
                    );
                    break;
                }
            }
        }
    });
}

fn spawn_exit_watcher(
    app: AppHandle,
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
    child: Arc<Mutex<Child>>,
    session_id: String,
) {
    thread::spawn(move || loop {
        let status = match child.lock() {
            Ok(mut process) => match process.try_wait() {
                Ok(exit_status) => exit_status,
                Err(error) => {
                    let _ = app.emit(
                        "terminal-output",
                        TerminalOutputEvent {
                            session_id: session_id.clone(),
                            stream: "system".to_string(),
                            text: format!("\n[系统] 无法检查终端状态: {}\n", error),
                        },
                    );
                    None
                }
            },
            Err(_) => None,
        };

        if let Some(exit_status) = status {
            if let Ok(mut all_sessions) = sessions.lock() {
                all_sessions.remove(&session_id);
            }

            let _ = app.emit(
                "terminal-exit",
                TerminalExitEvent {
                    session_id: session_id.clone(),
                    exit_code: exit_status.code(),
                },
            );
            break;
        }

        thread::sleep(Duration::from_millis(250));
    });
}

#[tauri::command]
pub fn start_terminal_session(
    app: AppHandle,
    state: State<'_, TerminalState>,
    shell: String,
    cwd: Option<String>,
) -> Result<TerminalSessionInfo, String> {
    let shell = normalize_shell(&shell)?;
    let (program, args) = create_shell_command(shell);
    let workdir = cwd
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| ".".to_string());

    let mut command = Command::new(&program);
    command
        .args(&args)
        .current_dir(&workdir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("启动终端失败: {}", error))?;

    let session_id = format!("{}-{}", shell, child.id());
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法获取终端标准输出".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法获取终端错误输出".to_string())?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "无法获取终端输入流".to_string())?;

    let child = Arc::new(Mutex::new(child));
    let stdin = Arc::new(Mutex::new(stdin));

    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.insert(
            session_id.clone(),
            TerminalSession {
                child: Arc::clone(&child),
                stdin: Arc::clone(&stdin),
            },
        );
    } else {
        return Err("终端状态初始化失败".to_string());
    }

    spawn_terminal_reader(stdout, app.clone(), session_id.clone(), "stdout");
    spawn_terminal_reader(stderr, app.clone(), session_id.clone(), "stderr");
    spawn_exit_watcher(
        app,
        Arc::clone(&state.sessions),
        Arc::clone(&child),
        session_id.clone(),
    );

    Ok(TerminalSessionInfo {
        session_id,
        shell: shell.to_string(),
        cwd: workdir,
    })
}

#[tauri::command]
pub fn write_terminal_input(
    state: State<'_, TerminalState>,
    session_id: String,
    input: String,
) -> Result<bool, String> {
    let stdin = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|_| "无法读取终端状态".to_string())?;
        sessions
            .get(&session_id)
            .map(|session| Arc::clone(&session.stdin))
            .ok_or_else(|| "终端会话不存在或已结束".to_string())?
    };

    let mut stdin = stdin.lock().map_err(|_| "无法访问终端输入流".to_string())?;

    stdin
        .write_all(input.as_bytes())
        .map_err(|error| format!("写入终端失败: {}", error))?;
    stdin
        .flush()
        .map_err(|error| format!("刷新终端输入失败: {}", error))?;

    Ok(true)
}

#[tauri::command]
pub fn stop_terminal_session(
    state: State<'_, TerminalState>,
    session_id: String,
) -> Result<bool, String> {
    let session = {
        let mut sessions = state
            .sessions
            .lock()
            .map_err(|_| "无法读取终端状态".to_string())?;
        sessions.remove(&session_id)
    };

    if let Some(session) = session {
        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
        return Ok(true);
    }

    Ok(false)
}
