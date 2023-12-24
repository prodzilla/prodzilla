use tokio::time::Instant;

use crate::probe::Probe;
use crate::http_probe::check_endpoint;


pub async fn schedule_probes(probes: Vec<Probe>) -> Result<(), Box<dyn std::error::Error>> {
    futures::future::join_all(probes.iter().map(|probe| schedule_probe(probe)))
        .await
        .into_iter()
        .collect::<Result<Vec<()>, Box<dyn std::error::Error>>>()?;

    Ok(())
}

pub async fn schedule_probe(probe: &Probe) -> Result<(), Box<dyn std::error::Error>> {

    let mut next_run_time = Instant::now() + std::time::Duration::from_secs(probe.schedule.initial_delay as u64);

    loop {
        let now = Instant::now();
        if now < next_run_time {
            tokio::time::sleep(next_run_time - now).await;
        }

        next_run_time += std::time::Duration::from_secs(probe.schedule.interval as u64);

        let _result = check_endpoint(probe).await;
    }
}