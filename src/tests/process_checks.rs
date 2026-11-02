use std::time::Duration;

// Een zombie op Linux is gestopt, maar wacht nog op reaping door zijn ouder/init.
#[cfg(target_os = "linux")]
pub(crate) fn linux_process_running(process_id: u32) -> bool {
    let stat = match std::fs::read_to_string(format!("/proc/{process_id}/stat")) {
        Ok(stat) => stat,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return false,
        Err(error) => panic!("Linux-processtatus lezen mislukt: {error}"),
    };
    let (_, fields) = stat
        .rsplit_once(") ")
        .expect("/proc-stat moet een processtatus bevatten");
    !matches!(fields.chars().next(), Some('Z' | 'X'))
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
