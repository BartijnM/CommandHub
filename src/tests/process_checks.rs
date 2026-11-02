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
async fn windows_process_running(process_id: u32) -> bool {
    let mut command = tokio::process::Command::new("tasklist");
    command.args(["/FO", "CSV", "/NH"]).kill_on_drop(true);
    let output = tokio::time::timeout(Duration::from_secs(3), command.output())
        .await
        .expect("tasklist moet tijdig afronden")
        .expect("tasklist moet starten");
    assert!(output.status.success(), "tasklist moet succesvol eindigen");
    let text = String::from_utf8_lossy(&output.stdout);
    let ids: Vec<u32> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split("\",\"")
                .nth(1)
                .expect("procesregel moet een PID bevatten")
                .trim_matches('"')
                .parse()
                .expect("PID moet numeriek zijn")
        })
        .collect();
    assert!(!ids.is_empty(), "proceslijst mag niet leeg zijn");
    ids.contains(&process_id)
}

pub(crate) async fn assert_stopped(process_id: u32) {
    assert!(process_id > 0, "proces-ID moet geldig zijn");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            #[cfg(windows)]
            let running = windows_process_running(process_id).await;
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

#[cfg(target_os = "linux")]
#[test]
fn linux_status_herkent_actief_en_ontbrekend_proces() {
    assert!(linux_process_running(std::process::id()));
    assert!(!linux_process_running(u32::MAX));
}

#[test]
fn linux_stat_herkent_verdwijnen_tijdens_lezen() {
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
fn linux_stat_onderscheidt_actief_zombie_en_exit() {
    // De procesnaam mag zelf spaties en haakjes bevatten.
    for (state, running) in [('R', true), ('S', true), ('Z', false), ('X', false)] {
        let stat = format!("123 (test (process)) {state} 1 2 3");
        assert_eq!(linux_stat_process_running(Ok(stat)).unwrap(), running);
    }
}

#[test]
fn linux_stat_verbergt_geen_toegangs_of_formaatfout() {
    let error = linux_stat_process_running(Err(std::io::ErrorKind::PermissionDenied.into()))
        .expect_err("geen toegang is geen bewijs van procesbeeindiging");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(linux_stat_process_running(Ok("ongeldige stat".into())).is_err());
    assert!(linux_stat_process_running(Ok("123 (test) ".into())).is_err());
}
