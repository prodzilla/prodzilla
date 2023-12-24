use tokio::time::Instant;

use crate::probe::Probe;
use crate::http_probe::check_endpoint;

async fn start_monitoring(probe: &Probe) -> Result<(), Box<dyn std::error::Error>> {

    let mut next_run_time = Instant::now() + std::time::Duration::from_secs(probe.schedule.initial_delay as u64);

    loop {
        let now = Instant::now();
        if now < next_run_time {
            tokio::time::sleep(next_run_time - now).await;
        }

        next_run_time += std::time::Duration::from_secs(probe.schedule.interval as u64);

        let probe_span = span!(parent: NO_PARENT, Level::INFO, "engine.probe",
            %probe.name,
            otel.name=probe.name,
            otel.status_code=?Status::Unset,
            otel.kind=?SpanKind::Consumer,
        );

        probe_span.follows_from(&parent_span);

        info!("Starting next probing session...");
        match probe.run().instrument(probe_span.clone()).await {
            Ok(_) => {
                probe_span
                    .record("otel.status_code", "Ok");
            },
            Err(err) => {
                probe_span
                    .record("otel.status_code", "Error")
                    .record("error", field::debug(&err));
            }
        }
    }

    Ok(())
}