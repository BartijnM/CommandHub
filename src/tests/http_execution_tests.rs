use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
    task::JoinHandle,
};

use crate::{
    app_state::AppState,
    config::{ConfigFile, ToolDefinition},
    execution::ExecutionService,
    registry::ToolRegistry,
};

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "commandhub-http-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).expect("unieke testmap moet aangemaakt worden");
        Self { directory }
    }

    fn ready(&self, index: usize) -> PathBuf {
        self.directory.join(format!("ready-{index}.json"))
    }
    fn release(&self) -> PathBuf {
        self.directory.join("release")
    }

    fn state(&self, count: usize) -> AppState {
        let executable = std::env::current_exe().expect("testbinary moet bestaan");
        let mut tools: Vec<_> = (0..count)
            .map(|index| ToolDefinition {
                name: format!("work_{index}"),
                description: "Gecontroleerde HTTP-testtool".to_owned(),
                command: executable.to_string_lossy().into_owned(),
                arguments: vec![
                    "--exact".into(),
                    "http_execution_tests::http_tool_helper".into(),
                    "--ignored".into(),
                    "--nocapture".into(),
                    "--skip".into(),
                    format!("commandhub-ready={}", self.ready(index).display()),
                    "--skip".into(),
                    format!("commandhub-release={}", self.release().display()),
                ],
                parameters: Vec::new(),
                timeout: Some(30_000),
            })
            .collect();
        tools.push(ToolDefinition {
            name: "git_version".into(),
            description: "Capaciteitsherstel controleren".into(),
            command: "git".into(),
            arguments: vec!["--version".into()],
            parameters: Vec::new(),
            timeout: Some(10_000),
        });
        let config = ConfigFile { tools };
        config
            .validate()
            .expect("testconfiguratie moet geldig zijn");
        AppState {
            registry: Arc::new(ToolRegistry::new(config)),
            execution: Arc::new(ExecutionService::new()),
        }
    }

    async fn pids(&self, index: usize) -> [u32; 2] {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                match std::fs::read(self.ready(index)) {
                    Ok(bytes) => {
                        if let Ok(pids) = serde_json::from_slice::<[u32; 2]>(&bytes) {
                            return pids;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => panic!("startbevestiging lezen mislukt: {error}"),
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("de HTTP-tool en zijn subprocess moeten starten")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

struct TestServer {
    address: SocketAddr,
    signal: Option<oneshot::Sender<()>>,
    task: JoinHandle<std::io::Result<()>>,
}

impl TestServer {
    async fn start(state: AppState) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("lokale testpoort moet beschikbaar zijn");
        let address = listener.local_addr().unwrap();
        let (signal, receiver) = oneshot::channel();
        let task = tokio::spawn(crate::server::serve(listener, state, async {
            let _ = receiver.await;
        }));
        Self {
            address,
            signal: Some(signal),
            task,
        }
    }

    fn signal_shutdown(&mut self) {
        self.signal
            .take()
            .expect("shutdownsignaal moet bestaan")
            .send(())
            .expect("server moet actief zijn");
    }

    async fn finish(&mut self) {
        tokio::time::timeout(Duration::from_secs(10), &mut self.task)
            .await
            .expect("server moet inclusief procesopruiming afsluiten")
            .expect("servertask mag niet panieken")
            .expect("server moet succesvol stoppen");
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(signal) = self.signal.take() {
            let _ = signal.send(());
        }
        self.task.abort();
    }
}

async fn post(address: SocketAddr, tool: &str) -> (u16, Value) {
    tokio::time::timeout(Duration::from_secs(20), async {
        let mut stream = TcpStream::connect(address).await.expect("TCP-client moet verbinden");
        // HTTP/1.0 met connection-close houdt de testclient klein: geen chunked decoder nodig.
        let request = format!("POST /tools/{tool} HTTP/1.0\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}");
        stream.write_all(request.as_bytes()).await.expect("request moet verstuurd worden");
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.expect("response moet leesbaar zijn");
        let boundary = response.windows(4).position(|bytes| bytes == b"\r\n\r\n").expect("HTTP-response moet headers bevatten");
        let headers = std::str::from_utf8(&response[..boundary]).unwrap();
        let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
        let body = serde_json::from_slice(&response[boundary + 4..]).expect("HTTP-body moet JSON zijn");
        (status, body)
    }).await.expect("HTTP-testrequest moet tijdig afronden")
}

#[tokio::test]
async fn vijf_echte_http_requests_begrenzen_en_capaciteit_herstellen() {
    let fixture = Fixture::new();
    let mut server = TestServer::start(fixture.state(4)).await;
    let mut requests = Vec::new();
    let mut pids = Vec::new();
    for index in 0..4 {
        let address = server.address;
        requests.push(tokio::spawn(async move {
            post(address, &format!("work_{index}")).await
        }));
        pids.extend(fixture.pids(index).await);
    }
    assert!(
        requests.iter().all(|request| !request.is_finished()),
        "vier HTTP-uitvoeringen moeten nog actief zijn"
    );
    let (status, body) =
        tokio::time::timeout(Duration::from_secs(2), post(server.address, "git_version"))
            .await
            .expect("vijfde request moet direct afgewezen worden");
    assert_eq!(status, 503);
    assert_eq!(body["code"], json!("capacity_exceeded"));
    std::fs::write(fixture.release(), b"stop").expect("testtools moeten vrijgegeven worden");
    for request in requests {
        let (status, body) = request.await.expect("HTTP-task moet afronden");
        assert_eq!(status, 200);
        assert_eq!(body["success"], json!(true));
    }
    let (status, body) = post(server.address, "git_version").await;
    assert_eq!(status, 200);
    assert_eq!(body["success"], json!(true));
    server.signal_shutdown();
    server.finish().await;
    for pid in pids {
        crate::process_checks::assert_stopped(pid).await;
    }
}

#[tokio::test]
async fn server_shutdown_stopt_actieve_http_procesboom() {
    let fixture = Fixture::new();
    let state = fixture.state(1);
    let execution = state.execution.clone();
    let mut server = TestServer::start(state).await;
    let address = server.address;
    let request = tokio::spawn(async move { post(address, "work_0").await });
    let pids = fixture.pids(0).await;
    server.signal_shutdown();
    let (status, body) = request.await.expect("actief request moet antwoord krijgen");
    assert_eq!(status, 503);
    assert_eq!(body["code"], json!("execution_cancelled"));
    server.finish().await;
    execution.shutdown().await;
    for pid in pids {
        crate::process_checks::assert_stopped(pid).await;
    }
    assert!(
        !matches!(
            tokio::time::timeout(Duration::from_secs(1), TcpStream::connect(address)).await,
            Ok(Ok(_))
        ),
        "afgesloten server mag geen nieuwe verbindingen accepteren"
    );
}

#[test]
#[ignore = "Wordt uitsluitend als geconfigureerd childproces gestart"]
fn http_tool_helper() {
    fn argument(prefix: &str) -> PathBuf {
        std::env::args()
            .find_map(|arg| arg.strip_prefix(prefix).map(PathBuf::from))
            .expect("helperargument moet bestaan")
    }
    let ready = argument("commandhub-ready=");
    let release = argument("commandhub-release=");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "http_execution_tests::subprocess_leaf",
            "--ignored",
            "--nocapture",
        ])
        .spawn()
        .expect("subprocess moet starten");
    std::fs::write(
        ready,
        serde_json::to_vec(&[std::process::id(), child.id()]).unwrap(),
    )
    .expect("PID's moeten opgeslagen worden");
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while !release
        .try_exists()
        .expect("vrijgavesignaal moet controleerbaar zijn")
    {
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("helper heeft geen vrijgavesignaal ontvangen");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.kill().expect("subprocess moet stoppen");
    child.wait().expect("subprocess moet gereaped worden");
}

#[test]
#[ignore = "Langlevend subprocess voor HTTP-procesboomtests"]
fn subprocess_leaf() {
    std::thread::sleep(Duration::from_secs(60));
}
