use std::{
    io,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::oneshot;
use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{OwnedSemaphorePermit, Semaphore},
};

use crate::{config::ToolDefinition, registry::ToolRegistry};

use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};

#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;

#[cfg(windows)]
use process_wrap::tokio::JobObject;

#[derive(Debug)]
// Representeert het resultaat van een uitgevoerde tool, inclusief successtatus, uitvoer, foutuitvoer, exitcode en duur van de uitvoering.
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration: Duration,
}

#[derive(Debug)]
// Representeert mogelijke fouten die kunnen optreden tijdens de uitvoering van een tool, zoals onbekende tools, overschrijding van capaciteit of timeouts.
pub enum ExecutionError {
    UnknownTool,
    ParametersNotSupported,
    InvalidParameters(&'static str),
    CapacityExceeded,
    TimedOut,
    OutputLimitExceeded,
    Cancelled,
    ProcessStartFailed(io::Error),
    OutputReadFailed(io::Error),
    ProcessWaitFailed(io::Error),
    ProcessKillFailed(io::Error),
}

// Definieert de maximale gelijktijdige uitvoeringen en de maximale uitvoerbytes voor de ExecutionService.
const MAX_CONCURRENT_EXECUTIONS: usize = 4;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

// Structuur die de uitvoer van een proces vastlegt, inclusief de gelezen bytes en de bijbehorende budgetpermissies.
struct CapturedOutput {
    bytes: Vec<u8>,
    _budget_permits: Vec<OwnedSemaphorePermit>,
}

// Tokio kill-on-drop stopt op Unix alleen de directe child. Deze bewaker stuurt
// bij future-drop ook een stopsignaal via de ProcessGroup-wrapper.
struct ProcessGuard(Box<dyn ChildWrapper>);

impl ProcessGuard {
    fn as_mut(&mut self) -> &mut dyn ChildWrapper {
        self.0.as_mut()
    }

    async fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        self.0.wait().await
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            let _ = self.0.start_kill();
        }
    }
}

// Service die verantwoordelijk is voor het uitvoeren van tools, het beheren van gelijktijdige uitvoeringen en het afhandelen van uitvoerbeperkingen.
pub struct ExecutionService {
    permits: Arc<Semaphore>,
    shutdown: tokio::sync::watch::Sender<bool>,
}

impl ExecutionService {
    pub fn new() -> Self {
        let (shutdown, _receiver) = tokio::sync::watch::channel(false);

        Self {
            permits: Arc::new(Semaphore::new(MAX_CONCURRENT_EXECUTIONS)),
            shutdown,
        }
    }

    // Probeert een permit te verkrijgen voor het starten van een nieuwe uitvoering. Als de capaciteit is bereikt, wordt een fout geretourneerd.
    fn try_acquire(&self) -> Result<OwnedSemaphorePermit, ExecutionError> {
        let shutting_down = self.shutdown.borrow();

        if *shutting_down {
            return Err(ExecutionError::Cancelled);
        }

        Arc::clone(&self.permits)
            .try_acquire_owned()
            .map_err(|_| ExecutionError::CapacityExceeded)
    }

    pub(crate) fn begin_shutdown(&self) {
        self.shutdown.send_replace(true);
    }

    pub(crate) async fn shutdown(&self) {
        self.begin_shutdown();

        let _permits = Arc::clone(&self.permits)
            .acquire_many_owned(MAX_CONCURRENT_EXECUTIONS as u32)
            .await
            .expect("de capaciteitssemaphore mag niet gesloten zijn");
    }

    // Start een proces op basis van de opgegeven tooldefinitie en verkrijgt een permit voor gelijktijdige uitvoering. Als het starten van het proces mislukt, wordt een fout geretourneerd.
    fn start_process(
        &self,
        tool: &ToolDefinition,
        arguments: &[String],
    ) -> Result<(Box<dyn ChildWrapper>, OwnedSemaphorePermit), ExecutionError> {
        let permit = self.try_acquire()?;
        let child = create_wrapped_command(tool, arguments)
            .spawn()
            .map_err(ExecutionError::ProcessStartFailed)?;

        Ok((child, permit))
    }

    // Voert een parameterloze tool uit op basis van de opgegeven naam en het geregistreerde tools. Het leest de uitvoer, wacht op voltooiing en retourneert het resultaat of een fout.
    pub(crate) async fn execute_parameterless(
        &self,
        registry: &ToolRegistry,
        name: &str,
    ) -> Result<ExecutionResult, ExecutionError> {
        let (_cancel_sender, cancel_receiver) = oneshot::channel();

        self.execute_parameterless_with_cancellation(registry, name, cancel_receiver)
            .await
    }

    pub(crate) async fn execute_parameterless_with_cancellation(
        &self,
        registry: &ToolRegistry,
        name: &str,
        cancellation: oneshot::Receiver<()>,
    ) -> Result<ExecutionResult, ExecutionError> {
        self.execute_logged(registry, name, None, cancellation).await
    }

    pub(crate) async fn execute_with_parameters(
        &self,
        registry: &ToolRegistry,
        name: &str,
        input: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<ExecutionResult, ExecutionError> {
        let (_cancel_sender, cancel_receiver) = oneshot::channel();

        self.execute_logged(registry, name, Some(input), cancel_receiver)
            .await
    }

    async fn execute_logged(
        &self,
        registry: &ToolRegistry,
        name: &str,
        input: Option<&serde_json::Map<String, serde_json::Value>>,
        cancellation: oneshot::Receiver<()>,
    ) -> Result<ExecutionResult, ExecutionError> {
        let started_at = Instant::now();

        let tool = registry.find(name);
        let tool_name = tool.map(|tool| tool.name.as_str()).unwrap_or("<unknown>");

        let result = match input {
            Some(input) => {
                self.execute_with_parameters_inner(tool, input, cancellation)
                    .await
            }
            None => self.execute_parameterless_inner(tool, cancellation).await,
        };

        match &result {
            Ok(output) => {
                tracing::info!(
                    tool = tool_name,
                    success = output.success,
                    exit_code = ?output.exit_code,
                    duration_ms = %started_at.elapsed().as_millis(),
                    "Tooluitvoering afgerond"
                );
            }
            Err(error) => {
                tracing::warn!(
                    tool = tool_name,
                    category = error.category(),
                    duration_ms = %started_at.elapsed().as_millis(),
                    "Tooluitvoering afgewezen of mislukt"
                );
            }
        }

        result
    }

    async fn execute_parameterless_inner(
        &self,
        tool: Option<&ToolDefinition>,
        cancellation: oneshot::Receiver<()>,
    ) -> Result<ExecutionResult, ExecutionError> {
        let tool = tool.ok_or(ExecutionError::UnknownTool)?;
        validate_parameterless_tool(tool)?;

        self.execute_tool_inner(tool, &tool.arguments, cancellation)
            .await
    }

    async fn execute_with_parameters_inner(
        &self,
        tool: Option<&ToolDefinition>,
        input: &serde_json::Map<String, serde_json::Value>,
        cancellation: oneshot::Receiver<()>,
    ) -> Result<ExecutionResult, ExecutionError> {
        let tool = tool.ok_or(ExecutionError::UnknownTool)?;

        let arguments = crate::validation::build_arguments(tool, input)
            .map_err(ExecutionError::InvalidParameters)?;

        self.execute_tool_inner(tool, &arguments, cancellation)
            .await
    }

    async fn execute_tool_inner(
        &self,
        tool: &ToolDefinition,
        arguments: &[String],
        cancellation: oneshot::Receiver<()>,
    ) -> Result<ExecutionResult, ExecutionError> {
        let started_at = Instant::now();
        let mut shutdown = self.shutdown.subscribe();
        let (child, _permit) = self.start_process(tool, arguments)?;
        let mut child = ProcessGuard(child);

        // Bepaal de timeout voor de uitvoering, standaard 10 seconden als geen specifieke timeout is opgegeven.
        let timeout = Duration::from_millis(tool.timeout.unwrap_or(10_000));
        let budget = Arc::new(Semaphore::new(MAX_OUTPUT_BYTES));

        // Deze future leest output en wacht op het proces; select! pollt hem in dezelfde taak.
        let uitvoering = async {
            let (stdout, stderr) = read_process_output(child.as_mut(), budget).await?;
            let status = child
                .wait()
                .await
                .map_err(ExecutionError::ProcessWaitFailed)?;

            Ok::<_, ExecutionError>((stdout, stderr, status))
        };

        // Wacht op de uitvoering met een timeout. Als de uitvoering succesvol is, retourneer het resultaat. Als er een fout optreedt of de timeout wordt overschreden, stop het proces en retourneer de fout.
        let afloop = tokio::select! {
            resultaat = tokio::time::timeout(timeout, uitvoering) => resultaat,
            Ok(()) = cancellation => {
                Ok(Err(ExecutionError::Cancelled))
            }
            Ok(()) = shutdown.changed() => {
                Ok(Err(ExecutionError::Cancelled))
            }
        };

        let (stdout, stderr, status) = match afloop {
            Ok(Ok(resultaat)) => resultaat,
            Ok(Err(fout)) => {
                stop_child(child.as_mut()).await?;
                return Err(fout);
            }
            Err(_) => {
                stop_child(child.as_mut()).await?;
                return Err(ExecutionError::TimedOut);
            }
        };

        // Bouw het ExecutionResult op basis van de verkregen uitvoer, status en duur van de uitvoering.
        Ok(ExecutionResult {
            success: status.success(),
            stdout: String::from_utf8_lossy(&stdout.bytes).into_owned(),
            stderr: String::from_utf8_lossy(&stderr.bytes).into_owned(),
            exit_code: status.code(),
            duration: started_at.elapsed(),
        })
    }
}

fn validate_parameterless_tool(tool: &ToolDefinition) -> Result<(), ExecutionError> {
    // Controleer of de tool parameters heeft of placeholders in de argumenten bevat. Als dat het geval is, retourneer een fout.
    let has_parameters = !tool.parameters.is_empty();
    let has_placeholders = tool
        .arguments
        .iter()
        .any(|argument| argument.contains('{') || argument.contains('}'));

    if has_parameters || has_placeholders {
        return Err(ExecutionError::ParametersNotSupported);
    }

    Ok(())
}

// Creëert een Command-object op basis van de opgegeven ToolDefinition, inclusief het instellen van de commandoregel, argumenten en standaardinvoer/-uitvoer/-fout.
fn create_command(tool: &ToolDefinition, arguments: &[String]) -> Command {
    let mut command = Command::new(tool.command.as_str());

    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    command
}

// Creëert een CommandWrap-object dat het proces op een platformonafhankelijke manier kan beheren, inclusief het instellen van procesgroepen of jobobjecten en het automatisch beëindigen van het proces bij het loslaten van de wrapper.
fn create_wrapped_command(tool: &ToolDefinition, arguments: &[String]) -> CommandWrap {
    let mut command = CommandWrap::from(create_command(tool, arguments));

    // Voeg platformafhankelijke wrappers toe om het proces te beheren en ervoor te zorgen dat het correct wordt beëindigd bij het loslaten van de wrapper.
    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());

    #[cfg(windows)]
    command.wrap(JobObject);

    command.wrap(KillOnDrop);
    command
}

// Leest de uitvoer van een asynchrone reader met een opgegeven budget voor het aantal bytes. Als de uitvoer boven het budget uitkomt, wordt een fout geretourneerd.
async fn read_capped<R>(
    mut reader: R,
    budget: Arc<Semaphore>,
) -> Result<CapturedOutput, ExecutionError>
where
    // De functie read_capped leest de uitvoer van een asynchrone reader (zoals stdout of stderr van een proces) en beperkt de hoeveelheid gelezen bytes op basis van een opgegeven budget. Als de uitvoer boven het budget uitkomt, wordt een fout geretourneerd.
    R: tokio::io::AsyncRead + Unpin,
{
    // Initialiseer een buffer om de gelezen bytes op te slaan en een vector om de verkregen budgetpermissies bij te houden.
    let mut bytes = Vec::new();
    let mut budget_permits = Vec::new();
    let mut buffer = [0_u8; 8192];

    // Lees de uitvoer in een lus totdat er geen bytes meer beschikbaar zijn of totdat het budget is overschreden.
    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .map_err(ExecutionError::OutputReadFailed)?;

        if bytes_read == 0 {
            break;
        }

        // Probeer het aantal gelezen bytes te verkrijgen van het budget. Als het budget is overschreden, retourneer een fout.
        let permit = Arc::clone(&budget)
            .try_acquire_many_owned(bytes_read as u32)
            .map_err(|_| ExecutionError::OutputLimitExceeded)?;

        bytes.extend_from_slice(&buffer[..bytes_read]);
        budget_permits.push(permit);
    }

    Ok(CapturedOutput {
        bytes,
        _budget_permits: budget_permits,
    })
}

// Leest de uitvoer van een proces (zowel stdout als stderr) met een gedeeld budget voor het aantal bytes. Als de gecombineerde uitvoer boven het budget uitkomt, wordt een fout geretourneerd.
async fn read_process_output(
    child: &mut dyn ChildWrapper,
    budget: Arc<Semaphore>,
) -> Result<(CapturedOutput, CapturedOutput), ExecutionError> {
    let stdout = child.stdout().take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stdout-pipe ontbreekt"))
    })?;
    let stderr = child.stderr().take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stderr-pipe ontbreekt"))
    })?;

    tokio::try_join!(
        read_capped(stdout, Arc::clone(&budget)),
        read_capped(stderr, budget)
    )
}

// Unix beëindigt de groep ook na afloop van de leider; Windows gebruikt het Job
// Object. Gecontroleerd stoppen wacht op de child en geeft beheerfouten terug.
async fn stop_child(child: &mut dyn ChildWrapper) -> Result<(), ExecutionError> {
    #[cfg(unix)]
    {
        // Ook na afloop van de groepsleider kunnen subprocessen nog leven.
        child
            .try_wait()
            .map_err(ExecutionError::ProcessWaitFailed)?;
        match child.start_kill() {
            Ok(()) => {}
            // ESRCH: de procesgroep is al verdwenen.
            Err(error) if error.raw_os_error() == Some(3) => {}
            Err(error) => return Err(ExecutionError::ProcessKillFailed(error)),
        }
        child
            .wait()
            .await
            .map_err(ExecutionError::ProcessWaitFailed)?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        if child
            .try_wait()
            .map_err(ExecutionError::ProcessWaitFailed)?
            .is_none()
        {
            let kill_future = child.kill();
            std::pin::Pin::from(kill_future)
                .await
                .map_err(ExecutionError::ProcessKillFailed)?;
        }

        Ok(())
    }
}

impl ExecutionError {
    fn category(&self) -> &'static str {
        match self {
            Self::UnknownTool => "unknown_tool",
            Self::ParametersNotSupported => "parameters_not_supported",
            Self::InvalidParameters(_) => "invalid_parameters",
            Self::CapacityExceeded => "capacity_exceeded",
            Self::TimedOut => "execution_timeout",
            Self::OutputLimitExceeded => "output_limit_exceeded",
            Self::Cancelled => "execution_cancelled",
            Self::ProcessStartFailed(_) => "process_start_failed",
            Self::OutputReadFailed(_) => "output_read_failed",
            Self::ProcessWaitFailed(_) => "process_wait_failed",
            Self::ProcessKillFailed(_) => "process_kill_failed",
        }
    }
}

// Implementeert de Display-trait voor ExecutionError om een leesbare foutmelding te genereren op basis van het type fout.
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTool => write!(formatter, "Onbekende tool"),
            Self::ParametersNotSupported => {
                write!(formatter, "Deze tool ondersteunt nog geen parameters")
            }
            Self::InvalidParameters(message) => write!(formatter, "{message}"),
            Self::CapacityExceeded => write!(formatter, "Uitvoercapaciteit bereikt"),
            Self::TimedOut => write!(formatter, "Uitvoering verlopen"),
            Self::OutputLimitExceeded => write!(formatter, "Uitvoerlimiet overschreden"),
            Self::Cancelled => write!(formatter, "Uitvoering geannuleerd"),
            Self::ProcessStartFailed(_) => write!(formatter, "Proces starten mislukt"),
            Self::OutputReadFailed(_) => write!(formatter, "Procesuitvoer lezen mislukt"),
            Self::ProcessWaitFailed(_) => write!(formatter, "Wachten op proces mislukt"),
            Self::ProcessKillFailed(_) => write!(formatter, "Proces beëindigen mislukt"),
        }
    }
}

// Implementeert de Error-trait voor ExecutionError om de bron van de fout te kunnen achterhalen, indien beschikbaar.
impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProcessStartFailed(error)
            | Self::OutputReadFailed(error)
            | Self::ProcessWaitFailed(error)
            | Self::ProcessKillFailed(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "tests/execution_tests.rs"]
mod tests;
