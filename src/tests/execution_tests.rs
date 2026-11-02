use tokio::io::AsyncWriteExt;

use super::*;
use crate::config::{ConfigFile, ParameterDefinition, ParameterType};

#[derive(Clone)]
struct LogBuffer {
    bytes: Arc<std::sync::Mutex<Vec<u8>>>,
}

impl std::io::Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes
            .lock()
            .map_err(|_| std::io::Error::other("logbuffer vergrendelen mislukt"))?
            .extend_from_slice(bytes);

        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

async fn run_logging_test_in_isolation(test_name: &str) -> bool {
    const CHILD_MARKER: &str = "COMMANDHUB_LOGGING_TEST_CHILD";

    if std::env::var(CHILD_MARKER).ok().as_deref() == Some(test_name) {
        return false;
    }

    let test_program = std::env::current_exe().expect("het testprogramma moet beschikbaar zijn");

    let mut command = tokio::process::Command::new(test_program);
    command
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_MARKER, test_name)
        .kill_on_drop(true);

    let output = tokio::time::timeout(Duration::from_secs(15), command.output())
        .await
        .expect("de geïsoleerde loggingtest moet tijdig afronden")
        .expect("de geïsoleerde loggingtest moet starten");

    assert!(
        output.status.success(),
        "geïsoleerde loggingtest mislukt:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    true
}

fn test_registry(parameters: Vec<ParameterDefinition>) -> ToolRegistry {
    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "git_versie".to_owned(),
            description: "Toon de geinstalleerde Git-versie".to_owned(),
            command: "git".to_owned(),
            arguments: vec!["--version".to_owned()],
            parameters,
            timeout: None,
        }],
    };

    ToolRegistry::new(config)
}

#[cfg(any(windows, unix))]
#[tokio::test]
// Deze test is bedoeld om te controleren of de platformwrapper correct werkt. Het start een proces met de wrapper en wacht op een succesvolle afloop.
async fn start_met_platformwrapper_en_wacht_op_succes() {
    let registry = test_registry(Vec::new());
    let tool = find_parameterless_tool(&registry, "git_versie")
        .expect("de ingestelde parameterloze tool moet bestaan");

    let mut command = create_wrapped_command(tool);
    let mut child = command
        .spawn()
        .expect("Het process met platformwrapper moet starten");

    let status = child
        .wait()
        .await
        .expect("op het proces met platformwrapper moet gewacht kunnen worden");

    assert!(status.success());
}

#[test]
#[ignore]
#[expect(
    clippy::zombie_processes,
    reason = "Deze helper laat bewust een subprocess achter om procesboomopruiming te testen"
)]
fn subprocess_helper_bij_timeout() {
    #[cfg(windows)]
    let mut subprocess = std::process::Command::new("ping")
        .args(["-n", "30", "127.0.0.1"])
        .spawn()
        .expect("het testsubproces moet starten");

    #[cfg(unix)]
    let mut subprocess = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("het testsubproces moet starten");

    let pid_file = std::env::args().find_map(|argument| {
        argument
            .strip_prefix("commandhub-pid-file=")
            .map(std::path::PathBuf::from)
    });

    if let Some(path) = pid_file {
        std::fs::write(path, subprocess.id().to_string())
            .expect("het subprocess-ID moet opgeslagen kunnen worden");
    }

    if std::env::args().any(|argument| argument == "commandhub-parent-exit") {
        return;
    }

    if std::env::args().any(|argument| argument == "commandhub-large-output") {
        use std::io::Write;

        let bytes = vec![b'x'; MAX_OUTPUT_BYTES + 1];
        let mut stdout = std::io::stdout().lock();

        stdout
            .write_all(&bytes)
            .expect("de helper moet output boven de limiet kunnen schrijven");

        stdout
            .flush()
            .expect("de helperoutput moet worden doorgestuurd");
    }

    subprocess
        .wait()
        .expect("op het testsubproces moet gewacht kunnen worden");
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn stopt_subproces_bij_timeout() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het huidige testprogramma moet bestaan");

    let pid_file =
        std::env::temp_dir().join(format!("commandhub-subprocess-{}.pid", std::process::id()));

    match std::fs::remove_file(&pid_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("oud PID-bestand verwijderen mislukt: {error}"),
    }

    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "procesboom_timeout".to_owned(),
            description: "Test opruiming van een subprocess bij timeout".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::subprocess_helper_bij_timeout".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
                "--skip".to_owned(),
                format!("commandhub-pid-file={}", pid_file.display()),
            ],
            parameters: Vec::new(),
            timeout: Some(1_500),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = tokio::time::timeout(
        Duration::from_secs(6),
        service.execute_parameterless(&registry, "procesboom_timeout"),
    )
    .await
    .expect("het proces en subprocess moeten tijdig worden beëindigd");

    assert!(matches!(resultaat, Err(ExecutionError::TimedOut)));

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na de procesboom-timeout weer vrij zijn")
        })
        .collect();

    let pid_text = std::fs::read_to_string(&pid_file)
        .expect("de helper moet het subprocess-ID hebben opgeslagen");

    let subprocess_id = pid_text
        .parse::<u32>()
        .expect("het PID-bestand moet een geldig proces-ID bevatten");

    std::fs::remove_file(&pid_file)
        .expect("het PID-bestand moet na het uitlezen verwijderd kunnen worden");

    assert!(
        subprocess_id > 0,
        "het subprocess-ID moet groter dan nul zijn"
    );

    crate::process_checks::assert_stopped(subprocess_id).await;
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn stopt_subproces_bij_outputoverschrijding() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het huidige testprogramma moet bestaan");

    let pid_file = std::env::temp_dir().join(format!(
        "commandhub-output-subprocess-{}.pid",
        std::process::id()
    ));

    match std::fs::remove_file(&pid_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("oud PID-bestand verwijderen mislukt: {error}"),
    }

    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "procesboom_output".to_owned(),
            description: "Test opruiming van een subprocess bij outputoverschrijding".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::subprocess_helper_bij_timeout".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
                "--skip".to_owned(),
                format!("commandhub-pid-file={}", pid_file.display()),
                "--skip".to_owned(),
                "commandhub-large-output".to_owned(),
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = tokio::time::timeout(
        Duration::from_secs(6),
        service.execute_parameterless(&registry, "procesboom_output"),
    )
    .await
    .expect("het proces en subprocess moeten tijdig worden beëindigd");

    assert!(matches!(
        resultaat,
        Err(ExecutionError::OutputLimitExceeded)
    ));

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na de procesboom-outputoverschrijding weer vrij zijn")
        })
        .collect();

    let pid_text = std::fs::read_to_string(&pid_file)
        .expect("de helper moet het subprocess-ID hebben opgeslagen");

    let subprocess_id = pid_text
        .parse::<u32>()
        .expect("het PID-bestand moet een geldig proces-ID bevatten");

    std::fs::remove_file(&pid_file)
        .expect("het PID-bestand moet na het uitlezen verwijderd kunnen worden");

    assert!(
        subprocess_id > 0,
        "het subprocess-ID moet groter dan nul zijn"
    );

    crate::process_checks::assert_stopped(subprocess_id).await;
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn stopt_subproces_als_parent_al_is_afgerond() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het huidige testprogramma moet bestaan");

    let pid_file = std::env::temp_dir().join(format!(
        "commandhub-parent-exit-subprocess-{}.pid",
        std::process::id()
    ));

    match std::fs::remove_file(&pid_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("oud PID-bestand verwijderen mislukt: {error}"),
    }

    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "procesboom_parent_exit".to_owned(),
            description: "Test opruiming nadat het directe child is afgerond".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::subprocess_helper_bij_timeout".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
                "--skip".to_owned(),
                format!("commandhub-pid-file={}", pid_file.display()),
                "--skip".to_owned(),
                "commandhub-parent-exit".to_owned(),
            ],
            parameters: Vec::new(),
            timeout: Some(1_500),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = tokio::time::timeout(
        Duration::from_secs(6),
        service.execute_parameterless(&registry, "procesboom_parent_exit"),
    )
    .await
    .expect("het proces en subprocess moeten tijdig worden beëindigd");

    assert!(matches!(resultaat, Err(ExecutionError::TimedOut)));

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na de procesboom-timeout weer vrij zijn")
        })
        .collect();

    let pid_text = std::fs::read_to_string(&pid_file)
        .expect("de helper moet het subprocess-ID hebben opgeslagen");

    let subprocess_id = pid_text
        .parse::<u32>()
        .expect("het PID-bestand moet een geldig proces-ID bevatten");

    std::fs::remove_file(&pid_file)
        .expect("het PID-bestand moet na het uitlezen verwijderd kunnen worden");

    assert!(
        subprocess_id > 0,
        "het subprocess-ID moet groter dan nul zijn"
    );

    crate::process_checks::assert_stopped(subprocess_id).await;
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn stopt_subproces_bij_annulering() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het huidige testprogramma moet bestaan");

    let pid_file = std::env::temp_dir().join(format!(
        "commandhub-cancel-subprocess-{}.pid",
        std::process::id()
    ));

    match std::fs::remove_file(&pid_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("oud PID-bestand verwijderen mislukt: {error}"),
    }

    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "procesboom_cancel".to_owned(),
            description: "Test procesboomopruiming bij annulering".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::subprocess_helper_bij_timeout".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
                "--skip".to_owned(),
                format!("commandhub-pid-file={}", pid_file.display()),
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let (sender, receiver) = tokio::sync::oneshot::channel();

    let resultaat = tokio::time::timeout(Duration::from_secs(6), async {
        let annulering = async {
            loop {
                if let Ok(text) = std::fs::read_to_string(&pid_file)
                    && text.parse::<u32>().is_ok()
                {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            sender
                .send(())
                .expect("het annuleringssignaal moet verstuurd worden");
        };

        let (uitkomst, ()) = tokio::join!(
            service.execute_parameterless_with_cancellation(
                &registry,
                "procesboom_cancel",
                receiver
            ),
            annulering
        );

        uitkomst
    })
    .await
    .expect("starten en annuleren moeten tijdig terugkeren");

    assert!(matches!(resultaat, Err(ExecutionError::Cancelled)));

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na de procesboom-annulering weer vrij zijn")
        })
        .collect();

    let pid_text = std::fs::read_to_string(&pid_file)
        .expect("de helper moet het subprocess-ID hebben opgeslagen");

    let subprocess_id = pid_text
        .parse::<u32>()
        .expect("het PID-bestand moet een geldig proces-ID bevatten");

    std::fs::remove_file(&pid_file)
        .expect("het PID-bestand moet na het uitlezen verwijderd kunnen worden");

    assert!(
        subprocess_id > 0,
        "het subprocess-ID moet groter dan nul zijn"
    );

    crate::process_checks::assert_stopped(subprocess_id).await;
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn stopt_subproces_bij_shutdown() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het huidige testprogramma moet bestaan");

    let pid_file = std::env::temp_dir().join(format!(
        "commandhub-shutdown-subprocess-{}.pid",
        std::process::id()
    ));

    match std::fs::remove_file(&pid_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("oud PID-bestand verwijderen mislukt: {error}"),
    }

    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "procesboom_shutdown".to_owned(),
            description: "Test procesboomopruiming bij shutdown".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::subprocess_helper_bij_timeout".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
                "--skip".to_owned(),
                format!("commandhub-pid-file={}", pid_file.display()),
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = tokio::time::timeout(Duration::from_secs(6), async {
        let shutdown = async {
            loop {
                if let Ok(text) = std::fs::read_to_string(&pid_file)
                    && text.parse::<u32>().is_ok()
                {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            service.shutdown().await;
        };

        let (uitkomst, ()) = tokio::join!(
            service.execute_parameterless(&registry, "procesboom_shutdown"),
            shutdown
        );

        uitkomst
    })
    .await
    .expect("starten en shutdown moeten tijdig terugkeren");

    assert!(matches!(resultaat, Err(ExecutionError::Cancelled)));

    assert_eq!(
        service.permits.available_permits(),
        MAX_CONCURRENT_EXECUTIONS
    );

    let pid_text = std::fs::read_to_string(&pid_file)
        .expect("de helper moet het subprocess-ID hebben opgeslagen");

    let subprocess_id = pid_text
        .parse::<u32>()
        .expect("het PID-bestand moet een geldig proces-ID bevatten");

    std::fs::remove_file(&pid_file)
        .expect("het PID-bestand moet na het uitlezen verwijderd kunnen worden");

    assert!(
        subprocess_id > 0,
        "het subprocess-ID moet groter dan nul zijn"
    );

    crate::process_checks::assert_stopped(subprocess_id).await;
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn stopt_subproces_bij_drop_van_uitvoeringsfuture() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het huidige testprogramma moet bestaan");

    let pid_file = std::env::temp_dir().join(format!(
        "commandhub-drop-subprocess-{}.pid",
        std::process::id()
    ));

    match std::fs::remove_file(&pid_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("oud PID-bestand verwijderen mislukt: {error}"),
    }

    let registry = ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "procesboom_drop".to_owned(),
            description: "Test procesboomopruiming bij droppen van de uitvoeringsfuture".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::subprocess_helper_bij_timeout".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
                "--skip".to_owned(),
                format!("commandhub-pid-file={}", pid_file.display()),
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    });
    let service = ExecutionService::new();

    tokio::time::timeout(Duration::from_secs(6), async {
        let uitvoering = service.execute_parameterless(&registry, "procesboom_drop");
        tokio::pin!(uitvoering);

        let subprocess_gestart = async {
            loop {
                if let Ok(text) = std::fs::read_to_string(&pid_file)
                    && text.parse::<u32>().is_ok()
                {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };

        tokio::select! {
            resultaat = &mut uitvoering => {
                panic!("uitvoering eindigde vóór de dropcontrole: {resultaat:?}");
            }
            () = subprocess_gestart => {}
        }
    })
    .await
    .expect("het subprocess moet tijdig starten");

    assert_eq!(
        service.permits.available_permits(),
        MAX_CONCURRENT_EXECUTIONS
    );

    let pid_text = std::fs::read_to_string(&pid_file)
        .expect("de helper moet het subprocess-ID hebben opgeslagen");

    let subprocess_id = pid_text
        .parse::<u32>()
        .expect("het PID-bestand moet een geldig proces-ID bevatten");

    std::fs::remove_file(&pid_file)
        .expect("het PID-bestand moet na het uitlezen verwijderd kunnen worden");

    assert!(
        subprocess_id > 0,
        "het subprocess-ID moet groter dan nul zijn"
    );

    crate::process_checks::assert_stopped(subprocess_id).await;
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn stopt_subproces_bij_afbreken_van_taak() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het huidige testprogramma moet bestaan");

    let pid_file = std::env::temp_dir().join(format!(
        "commandhub-abort-subprocess-{}.pid",
        std::process::id()
    ));

    match std::fs::remove_file(&pid_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("oud PID-bestand verwijderen mislukt: {error}"),
    }

    let registry = ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "procesboom_abort".to_owned(),
            description: "Test procesboomopruiming bij afbreken van de taak".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::subprocess_helper_bij_timeout".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
                "--skip".to_owned(),
                format!("commandhub-pid-file={}", pid_file.display()),
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    });
    let service = Arc::new(ExecutionService::new());
    let task_service = Arc::clone(&service);

    let task = tokio::spawn(async move {
        task_service
            .execute_parameterless(&registry, "procesboom_abort")
            .await
    });

    let gestart = tokio::time::timeout(Duration::from_secs(6), async {
        loop {
            if let Ok(text) = std::fs::read_to_string(&pid_file)
                && text.parse::<u32>().is_ok()
            {
                break;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    task.abort();

    let task_error = task
        .await
        .expect_err("de afgebroken taak mag niet normaal afronden");

    assert!(
        task_error.is_cancelled(),
        "verwachtte taakannulering, kreeg: {task_error}"
    );

    gestart.expect("het subprocess moet vóór het afbreken gestart zijn");

    assert_eq!(
        service.permits.available_permits(),
        MAX_CONCURRENT_EXECUTIONS
    );

    let pid_text = std::fs::read_to_string(&pid_file)
        .expect("de helper moet het subprocess-ID hebben opgeslagen");

    let subprocess_id = pid_text
        .parse::<u32>()
        .expect("het PID-bestand moet een geldig proces-ID bevatten");

    std::fs::remove_file(&pid_file)
        .expect("het PID-bestand moet na het uitlezen verwijderd kunnen worden");

    assert!(
        subprocess_id > 0,
        "het subprocess-ID moet groter dan nul zijn"
    );

    crate::process_checks::assert_stopped(subprocess_id).await;
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn annuleert_uitvoering_bij_expliciet_signaal() {
    #[cfg(windows)]
    let (command, arguments) = (
        "ping",
        vec!["-n".to_owned(), "30".to_owned(), "127.0.0.1".to_owned()],
    );

    #[cfg(unix)]
    let (command, arguments) = ("sleep", vec!["30".to_owned()]);

    let registry = ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "annuleren".to_owned(),
            description: "Test expliciete annulering".to_owned(),
            command: command.to_owned(),
            arguments,
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    });
    let service = ExecutionService::new();
    let (sender, receiver) = tokio::sync::oneshot::channel();

    sender
        .send(())
        .expect("het annuleringssignaal moet verstuurd worden");

    let resultaat = tokio::time::timeout(
        Duration::from_secs(6),
        service.execute_parameterless_with_cancellation(&registry, "annuleren", receiver),
    )
    .await
    .expect("annulering moet tijdig terugkeren");

    assert!(
        matches!(resultaat, Err(ExecutionError::Cancelled)),
        "verwachtte annulering, kreeg: {resultaat:?}"
    );

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na annulering weer vrij zijn")
        })
        .collect();
}

#[tokio::test]
async fn voert_tool_uit_als_annuleringszender_wordt_gedropt() {
    let registry = test_registry(Vec::new());
    let service = ExecutionService::new();
    let (sender, receiver) = tokio::sync::oneshot::channel();

    drop(sender);

    let resultaat = tokio::time::timeout(
        Duration::from_secs(6),
        service.execute_parameterless_with_cancellation(&registry, "git_versie", receiver),
    )
    .await
    .expect("de uitvoering moet tijdig terugkeren")
    .expect("een gesloten annuleringskanaal mag de uitvoering niet annuleren");

    assert!(resultaat.success);
    assert_eq!(resultaat.exit_code, Some(0));
    assert!(!resultaat.stdout.is_empty());
}

#[tokio::test]
async fn stopt_geen_al_afgerond_proces() {
    let registry = test_registry(Vec::new());
    let tool = find_parameterless_tool(&registry, "git_versie")
        .expect("de ingestelde parameterloze tool moet bestaan");

    let mut command = create_wrapped_command(tool);
    let mut child = command.spawn().expect("het proces moet starten");

    let status = child
        .wait()
        .await
        .expect("op het proces moet gewacht kunnen worden");

    assert!(status.success());

    stop_child(child.as_mut())
        .await
        .expect("een al afgerond proces hoeft niet opnieuw te worden beëindigd");
}

#[test]
// Deze test controleert of de functie find_parameterless_tool correct een tool zonder parameters kan vinden.
fn vindt_parameterloze_tool() {
    let registry = test_registry(Vec::new());

    let tool = find_parameterless_tool(&registry, "git_versie")
        .expect("de ingestelde tool zonder parameters moet worden gevonden");

    assert_eq!(tool.command, "git");
}

#[test]
// Deze test controleert of de functie find_parameterless_tool correct een foutmelding geeft bij een onbekende tool.
fn weigert_onbekende_tool() {
    let registry = test_registry(Vec::new());

    let resultaat = find_parameterless_tool(&registry, "onbekende_tool");

    assert!(matches!(resultaat, Err(ExecutionError::UnknownTool)));
}

#[test]
// Deze test controleert of de functie find_parameterless_tool correct een foutmelding geeft bij een tool met parameters.
fn weigert_tool_met_parameters() {
    let registry = test_registry(vec![ParameterDefinition {
        name: "pad".to_owned(),
        kind: ParameterType::String,
        required: true,
    }]);

    let resultaat = find_parameterless_tool(&registry, "git_versie");

    assert!(matches!(
        resultaat,
        Err(ExecutionError::ParametersNotSupported)
    ));
}

#[tokio::test]
async fn shutdown_wacht_tot_capaciteit_is_vrijgegeven() {
    let service = ExecutionService::new();
    let permit = service
        .try_acquire()
        .expect("capaciteit moet beschikbaar zijn");

    let shutdown = service.shutdown();
    tokio::pin!(shutdown);

    tokio::select! {
        biased;

        () = &mut shutdown => {
            panic!("shutdown mag niet afronden zolang een permit bezet is");
        }
        () = std::future::ready(()) => {}
    }

    assert!(*service.shutdown.borrow());

    drop(permit);

    tokio::time::timeout(Duration::from_secs(1), shutdown)
        .await
        .expect("shutdown moet na capaciteitsvrijgave afronden");
}

#[test]
fn weigert_nieuwe_uitvoering_na_begin_shutdown() {
    let service = ExecutionService::new();

    service.begin_shutdown();

    assert!(matches!(
        service.try_acquire(),
        Err(ExecutionError::Cancelled)
    ));

    assert_eq!(
        service.permits.available_permits(),
        MAX_CONCURRENT_EXECUTIONS
    );
}

#[test]
// Deze test controleert of de ExecutionService correct een foutmelding geeft wanneer de maximale uitvoercapaciteit is bereikt.
fn weigert_extra_uitvoering_bij_volledige_capaciteit() {
    let service = ExecutionService::new();
    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet beschikbaar zijn")
        })
        .collect();

    assert!(matches!(
        service.try_acquire(),
        Err(ExecutionError::CapacityExceeded)
    ));
}

#[test]
// Deze test controleert of de ExecutionService correct de capaciteit vrijgeeft wanneer het starten van een proces mislukt.
fn geeft_capaciteit_vrij_als_starten_mislukt() {
    let service = ExecutionService::new();
    let tool = ToolDefinition {
        name: "ongeldig".to_owned(),
        description: "Test een mislukte processtart".to_owned(),
        command: "\0".to_owned(),
        arguments: Vec::new(),
        parameters: Vec::new(),
        timeout: None,
    };

    assert!(matches!(
        service.start_process(&tool),
        Err(ExecutionError::ProcessStartFailed(_))
    ));

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na de fout vrij zijn")
        })
        .collect();
}

#[tokio::test]
// Deze test controleert of de ExecutionService correct een proces start en wacht op een succesvolle afloop.
async fn start_proces_en_wacht_op_succesvolle_afloop() {
    let service = ExecutionService::new();
    let registry = test_registry(Vec::new());

    let resultaat = service
        .execute_parameterless(&registry, "git_versie")
        .await
        .expect("de git-testtool moet succesvol uitgevoerd worden");

    assert!(resultaat.success);
    assert_eq!(resultaat.exit_code, Some(0));
    assert!(!resultaat.stdout.is_empty());
    assert!(resultaat.stderr.is_empty());
}

#[tokio::test]
// Deze test controleert of de ExecutionService correct een resultaat geeft bij een proces met een niet-succesvolle exitcode.
async fn geeft_resultaat_bij_niet_succesvolle_exitcode() {
    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "git_fout".to_owned(),
            description: "Test een niet-succesvolle exitcode".to_owned(),
            command: "git".to_owned(),
            arguments: vec!["--invalid-commandhub-test".to_owned()],
            parameters: Vec::new(),
            timeout: None,
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = service
        .execute_parameterless(&registry, "git_fout")
        .await
        .expect("een proces met een foutieve exitcode moet een resultaat geven");

    assert!(!resultaat.success);
    assert!(matches!(resultaat.exit_code, Some(code) if code != 0));
    assert!(!resultaat.stderr.is_empty());
}

#[cfg(any(windows, unix))]
#[tokio::test]
// Deze test controleert of de ExecutionService correct een timeout geeft bij een proces dat te lang duurt.
async fn geeft_timeout_als_proces_te_lang_duurt() {
    #[cfg(windows)]
    let (command, arguments) = (
        "ping",
        vec!["-n".to_owned(), "5".to_owned(), "127.0.0.1".to_owned()],
    );

    #[cfg(unix)]
    let (command, arguments) = ("sleep", vec!["5".to_owned()]);

    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "langzaam".to_owned(),
            description: "Test de proces-timeout".to_owned(),
            command: command.to_owned(),
            arguments,
            parameters: Vec::new(),
            timeout: Some(100),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = service.execute_parameterless(&registry, "langzaam").await;

    assert!(
        matches!(resultaat, Err(ExecutionError::TimedOut)),
        "verwachtte een timeout, kreeg: {resultaat:?}"
    );

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na een timeout weer vrij zijn")
        })
        .collect();
}

#[tokio::test]
// Deze test controleert of de ExecutionService correct een foutmelding geeft bij een proces dat te veel uitvoer genereert.
async fn stopt_proces_bij_outputoverschrijding() {
    let testprogramma = std::env::current_exe()
        .expect("het pad naar het huidige testprogramma moet beschikbaar zijn");

    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "grote_output".to_owned(),
            description: "Test het stoppen bij te grote uitvoer".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::schrijft_output_boven_de_limiet".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = service
        .execute_parameterless(&registry, "grote_output")
        .await;

    assert!(
        matches!(resultaat, Err(ExecutionError::OutputLimitExceeded)),
        "verwachtte een outputlimietfout, kreeg: {resultaat:?}"
    );

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("capaciteit moet na outputoverschrijding vrij zijn")
        })
        .collect();
}

#[tokio::test]
// Deze test controleert of de read_capped functie correct uitvoer onder de limiet kan lezen.
async fn leest_output_binnen_de_limiet() {
    let (mut schrijver, lezer) = tokio::io::duplex(16);
    schrijver
        .write_all(b"uitvoer")
        .await
        .expect("testuitvoer moet geschreven worden");
    drop(schrijver);

    let budget = Arc::new(Semaphore::new(MAX_OUTPUT_BYTES));
    let uitvoer = read_capped(lezer, budget)
        .await
        .expect("uitvoer onder de limiet moet worden gelezen");

    assert_eq!(uitvoer.bytes, b"uitvoer");
}

#[tokio::test]
// Deze test controleert of de read_capped functie correct een foutmelding geeft wanneer de gecombineerde uitvoer van stdout en stderr boven de limiet uitkomt.
async fn begrenst_stdout_en_stderr_met_een_gedeeld_budget() {
    let (mut stdout_writer, stdout_reader) = tokio::io::duplex(16);
    let (mut stderr_writer, stderr_reader) = tokio::io::duplex(16);

    stdout_writer
        .write_all(b"123")
        .await
        .expect("stdout-testdata moet geschreven worden");
    stderr_writer
        .write_all(b"456")
        .await
        .expect("stderr-testdata moet geschreven worden");
    drop(stdout_writer);
    drop(stderr_writer);

    let budget = Arc::new(Semaphore::new(5));
    let resultaat = tokio::try_join!(
        read_capped(stdout_reader, Arc::clone(&budget)),
        read_capped(stderr_reader, budget)
    );

    assert!(matches!(
        resultaat,
        Err(ExecutionError::OutputLimitExceeded)
    ));
}

#[tokio::test]
// Deze test controleert of de read_capped functie correct een foutmelding geeft wanneer de uitvoer boven de limiet uitkomt.
async fn weigert_output_boven_de_limiet() {
    let (mut schrijver, lezer) = tokio::io::duplex(16);
    schrijver
        .write_all(b"12345")
        .await
        .expect("testuitvoer moet geschreven worden");
    drop(schrijver);

    let budget = Arc::new(Semaphore::new(4));
    let resultaat = read_capped(lezer, budget).await;

    assert!(matches!(
        resultaat,
        Err(ExecutionError::OutputLimitExceeded)
    ));
}

#[test]
#[ignore]
// Deze test is bedoeld om te controleren of de ExecutionService correct een foutmelding geeft bij een proces dat te veel uitvoer genereert. Het wordt genegeerd omdat het een groot aantal bytes schrijft en daardoor de testuitvoer kan vervuilen.
fn schrijft_output_boven_de_limiet() {
    use std::io::Write;

    let bytes = vec![b'x'; MAX_OUTPUT_BYTES + 1];
    std::io::stdout()
        .lock()
        .write_all(&bytes)
        .expect("de testuitvoer moet geschreven worden");
}

#[tokio::test]
async fn volle_capaciteit_geeft_http_503() {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;

    let service = Arc::new(ExecutionService::new());

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service
                .try_acquire()
                .expect("de vier uitvoeringsplekken moeten beschikbaar zijn")
        })
        .collect();

    let state = crate::app_state::AppState {
        registry: Arc::new(test_registry(Vec::new())),
        execution: Arc::clone(&service),
    };

    let response = tokio::time::timeout(
        Duration::from_secs(2),
        crate::routes::router(state).oneshot(
            Request::builder()
                .method("POST")
                .uri("/tools/git_versie")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("het testrequest moet geldig zijn"),
        ),
    )
    .await
    .expect("volle capaciteit moet zonder wachtrij worden afgewezen")
    .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("de foutresponse moet leesbaar zijn");

    let actual: serde_json::Value =
        serde_json::from_slice(&body).expect("de foutresponse moet JSON zijn");

    assert_eq!(
        actual,
        json!({
            "code": "capacity_exceeded",
            "message": "Uitvoercapaciteit bereikt"
        })
    );
}

#[tokio::test]
async fn logt_onbekende_tool_zonder_clientwaarde() {
    if run_logging_test_in_isolation("execution::tests::logt_onbekende_tool_zonder_clientwaarde")
        .await
    {
        return;
    }

    use tracing::instrument::WithSubscriber;

    let buffer = LogBuffer {
        bytes: Arc::new(std::sync::Mutex::new(Vec::new())),
    };
    let writer = buffer.clone();

    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || writer.clone())
        .finish();

    let registry = test_registry(Vec::new());
    let service = ExecutionService::new();
    let client_value = "niet_loggen_geheime_testwaarde_7391";

    let result = service
        .execute_parameterless(&registry, client_value)
        .with_subscriber(subscriber)
        .await;

    assert!(matches!(result, Err(ExecutionError::UnknownTool)));

    let bytes = buffer
        .bytes
        .lock()
        .expect("de logbuffer moet leesbaar zijn")
        .clone();

    let logs = String::from_utf8(bytes).expect("de opgevangen logs moeten UTF-8 zijn");

    assert!(logs.contains("Tooluitvoering afgewezen of mislukt"));
    assert!(logs.contains("tool=\"<unknown>\""));
    assert!(logs.contains("category=\"unknown_tool\""));
    assert!(logs.contains("duration_ms="));
    assert!(!logs.contains(client_value));
}

#[tokio::test]
async fn logt_resultaten_zonder_argumenten_of_procesoutput() {
    if run_logging_test_in_isolation(
        "execution::tests::logt_resultaten_zonder_argumenten_of_procesoutput",
    )
    .await
    {
        return;
    }

    use tracing::instrument::WithSubscriber;

    let buffer = LogBuffer {
        bytes: Arc::new(std::sync::Mutex::new(Vec::new())),
    };
    let writer = buffer.clone();

    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || writer.clone())
        .finish();

    let secret_marker = "commandhub-geheime-testwaarde-7391";
    let success_registry = test_registry(Vec::new());

    let failure_registry = ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "git_fout".to_owned(),
            description: "Test veilige logging van procesafloop".to_owned(),
            command: "git".to_owned(),
            arguments: vec![format!("--{secret_marker}")],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    });

    let service = ExecutionService::new();

    let (success, failure) = (async {
        let success = service
            .execute_parameterless(&success_registry, "git_versie")
            .await
            .expect("de Git-versie moet opgevraagd kunnen worden");

        let failure = service
            .execute_parameterless(&failure_registry, "git_fout")
            .await
            .expect("een niet-nul exitcode moet een resultaat geven");

        (success, failure)
    })
    .with_subscriber(subscriber)
    .await;

    assert!(success.success);
    assert!(!success.stdout.trim().is_empty());

    assert!(!failure.success);
    assert!(failure.stderr.contains(secret_marker));

    let bytes = buffer
        .bytes
        .lock()
        .expect("de logbuffer moet leesbaar zijn")
        .clone();

    let logs = String::from_utf8(bytes).expect("de opgevangen logs moeten UTF-8 zijn");

    assert!(
        logs.contains("tool=\"git_versie\""),
        "de verwachte toolnaam ontbreekt; opgevangen logs:\n{logs}"
    );
    assert!(logs.contains("tool=\"git_fout\""));
    assert!(logs.contains("success=true"));
    assert!(logs.contains("success=false"));
    assert!(logs.contains("exit_code="));
    assert!(logs.contains("duration_ms="));

    assert!(!logs.contains("--version"));
    assert!(!logs.contains(success.stdout.trim()));
    assert!(!logs.contains(secret_marker));
}
