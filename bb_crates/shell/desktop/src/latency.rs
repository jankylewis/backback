use crate::ExplorerView;
use gpui::Window;

impl ExplorerView {
    pub(crate) fn record_keypress(&mut self, window: &mut Window) {
        if self.keypresses == 0 {
            self.latency_before = Some(window.input_latency_snapshot());
        }
        self.keypresses = self.keypresses.saturating_add(1);
        if self.keypresses == 500 {
            let Some(before): Option<gpui::profiler::InputLatencySnapshot> =
                self.latency_before.take()
            else {
                return;
            };
            window.on_next_frame(move |window: &mut Window, _: &mut gpui::App| {
                window.on_next_frame(move |window: &mut Window, _: &mut gpui::App| {
                    let mut after: gpui::profiler::InputLatencySnapshot = window.input_latency_snapshot();
                    if let Err(error) = after.latency_histogram.subtract(&before.latency_histogram) {
                        eprintln!("editor_latency_error={error}");
                        return;
                    }
                    let samples: u64 = after.latency_histogram.len();
                    let p50: f64 = std::time::Duration::from_nanos(after.latency_histogram.value_at_quantile(0.50)).as_secs_f64() * 1000.0;
                    let p95: f64 = std::time::Duration::from_nanos(after.latency_histogram.value_at_quantile(0.95)).as_secs_f64() * 1000.0;
                    let p99: f64 = std::time::Duration::from_nanos(after.latency_histogram.value_at_quantile(0.99)).as_secs_f64() * 1000.0;
                    eprintln!("editor_latency keypresses=500 samples={samples} p50_ms={p50:.3} p95_ms={p95:.3} p99_ms={p99:.3}");
                });
            });
        }
    }
}
