use std::time::Duration;

// Een zombie op Linux is gestopt, maar wacht nog op reaping door zijn ouder/init.
#[cfg(target_os = "linux")]
pub(crate) fn linux_process_running(process_id: u32) -> bool {
    linux_stat_process_running(std::fs::read_to_string(format!("/proc/{process_id}/stat")))
        .unwrap_or_else(|error| panic!("Linux-processtatus lezen mislukt: {error}"))
}

// Los van /proc testbaar, ook op Windows: verdwijnen tussen open() en read()
// kan op Linux ESRCH opleveren in plaats van ENOENT.
fn linux_stat_process_running(result: std::io::Result<String>) -> std::io::Result<bool> {
    const ESRCH: i32 = 3;
    let stat = match result {
        Ok(stat) => stat,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.raw_os_error() == Some(ESRCH) =>
        {
            return Ok(false);
        }
        Err(error) => return Err(error),
    };
    let (_, fields) = stat.rsplit_once(") ").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "processtatus ontbreekt")
    })?;
    let state = fields.chars().next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "processtatus is leeg")
    })?;
    Ok(!matches!(state, 'Z' | 'X'))
}

#[cfg(windows)]
fn windows_process_running(process_id: u32) -> std::io::Result<bool> {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::{ERROR_INVALID_PARAMETER, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    };

    assert!(process_id > 0, "proces-ID moet geldig zijn");
    // SAFETY: geldig positief PID; alleen het recht om processtatus af te wachten.
    let raw = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, process_id) };
    if raw.is_null() {
        let error = std::io::Error::last_os_error();
        return if error.raw_os_error() == Some(ERROR_INVALID_PARAMETER as i32) {
            Ok(false)
        } else {
            Err(error)
        };
    }
    // SAFETY: OpenProcess leverde een uniek owned handle; Drop sluit het ook bij fouten.
    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    // SAFETY: het handle blijft geldig tijdens deze niet-blokkerende statuscontrole.
    match unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) } {
        WAIT_OBJECT_0 => Ok(false),
        WAIT_TIMEOUT => Ok(true),
        WAIT_FAILED => Err(std::io::Error::last_os_error()),
        _ => Err(std::io::Error::other("onverwachte Windows-processtatus")),
    }
}

pub(crate) async fn assert_stopped(process_id: u32) {
    assert!(process_id > 0, "proces-ID moet geldig zijn");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            #[cfg(windows)]
            let running = windows_process_running(process_id)
                .expect("Windows-processtatus moet leesbaar zijn");
            #[cfg(target_os = "linux")]
            let running = linux_process_running(process_id);
            #[cfg(not(any(windows, target_os = "linux")))]
            compile_error!("Voeg voor dit testplatform een processtatuscontrole toe");
            if !running {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("proces {process_id} draait nog na vijf seconden"));
}

#[cfg(windows)]
#[tokio::test]
async fn windows_status_detects_running_and_stopped_process() {
    assert!(windows_process_running(std::process::id()).unwrap());
    assert!(!windows_process_running(u32::MAX).unwrap());
    let mut child = tokio::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "http_execution_tests::subprocess_leaf",
            "--ignored",
        ])
        .kill_on_drop(true)
        .spawn()
        .expect("testproces moet starten");
    let pid = child.id().unwrap();
    assert!(windows_process_running(pid).unwrap());
    child.kill().await.expect("testproces moet stoppen");
    assert_stopped(pid).await;
}

#[cfg(target_os = "linux")]
#[test]
fn linux_status_detects_running_and_missing_process() {
    assert!(linux_process_running(std::process::id()));
    assert!(!linux_process_running(u32::MAX));
}

#[test]
fn linux_stat_handles_process_disappearing_during_read() {
    for error in [
        std::io::Error::from(std::io::ErrorKind::NotFound),
        std::io::Error::from_raw_os_error(3),
    ] {
        assert!(
            !linux_stat_process_running(Err(error)).expect("verdwenen proces geldt als gestopt")
        );
    }
}

#[test]
fn linux_stat_distinguishes_running_zombie_and_exited_states() {
    // De procesnaam mag zelf spaties en haakjes bevatten.
    for (state, running) in [('R', true), ('S', true), ('Z', false), ('X', false)] {
        let stat = format!("123 (test (process)) {state} 1 2 3");
        assert_eq!(linux_stat_process_running(Ok(stat)).unwrap(), running);
    }
}

#[test]
fn linux_stat_preserves_access_and_format_errors() {
    let error = linux_stat_process_running(Err(std::io::ErrorKind::PermissionDenied.into()))
        .expect_err("geen toegang is geen bewijs van procesbeeindiging");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(linux_stat_process_running(Ok("ongeldige stat".into())).is_err());
    assert!(linux_stat_process_running(Ok("123 (test) ".into())).is_err());
}
