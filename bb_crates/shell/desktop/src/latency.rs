use crate::ExplorerView;
use gpui::Window;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

static HIGHLIGHT_NS: AtomicU64 = AtomicU64::new(0);
static HIGHLIGHT_CALLS: AtomicU64 = AtomicU64::new(0);
static DRIVEN: AtomicBool = AtomicBool::new(false);
static PARSE_NS: AtomicU64 = AtomicU64::new(0);
static PARSE_JOBS: AtomicU64 = AtomicU64::new(0);

/// One background parse finished in `took`.
pub(crate) fn record_parse(took: Duration) {
    PARSE_NS.fetch_add(nanos(took), Ordering::Relaxed);
    PARSE_JOBS.fetch_add(1, Ordering::Relaxed);
}

/// Keys to send through the window's own input path once a file opens:
/// `BB_KEYBENCH=<n>` types `n` chars mid-file and prints per-key work;
/// `BB_KEYS="x cmd-s"` sends those keys. `BB_KEYS_QUIT=1` quits after.
pub(crate) fn drive(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut gpui::Context<ExplorerView>,
) {
    let bench: Option<usize> = std::env::var("BB_KEYBENCH")
        .ok()
        .and_then(|count: String| count.parse::<usize>().ok());
    let keys: Vec<String> = match bench {
        Some(count) => vec![String::from("x"); count],
        None => std::env::var("BB_KEYS")
            .map(|list: String| list.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default(),
    };
    if keys.is_empty() || DRIVEN.swap(true, Ordering::SeqCst) {
        return;
    }
    if let (Some(_), Some(doc)) = (bench, view.tabs.active_mut()) {
        park_mid_file(doc);
    }
    cx.spawn_in(
        window,
        async move |_: gpui::WeakEntity<ExplorerView>, cx: &mut gpui::AsyncWindowContext| {
            let mut work: Vec<u64> = Vec::new();
            for key in keys {
                work.extend(send_key(&key, cx).await);
            }
            report_keys(&mut work);
            if std::env::var("BB_KEYS_QUIT").is_ok_and(|value: String| value == "1") {
                cx.update(|_: &mut Window, cx: &mut gpui::App| cx.quit())
                    .ok();
            }
        },
    )
    .detach();
}

/// Put the cursor at the start of the middle line (the bench types there).
fn park_mid_file(doc: &mut bb_text::Document) {
    let at: bb_text::CharIdx = doc
        .line_start(doc.len_lines() / 2)
        .unwrap_or(bb_text::CharIdx::new(0));
    doc.set_cursor(at).ok();
}

/// Send one key at window level (the key handler updates the view itself) and
/// return its UI work: dispatch time plus the next frame's highlight time.
async fn send_key(key: &str, cx: &mut gpui::AsyncWindowContext) -> Option<u64> {
    cx.background_executor()
        .timer(Duration::from_millis(30))
        .await;
    let paint_before: u64 = HIGHLIGHT_NS.load(Ordering::Relaxed);
    let Ok(stroke): Result<gpui::Keystroke, _> =
        gpui::Keystroke::parse_with_os(key, gpui::OperatingSystem::Mac)
    else {
        eprintln!("bb_keys bad_key={key}");
        return None;
    };
    let took: Result<u64, _> = cx.update(|window: &mut Window, cx: &mut gpui::App| {
        let started: std::time::Instant = std::time::Instant::now();
        window.dispatch_keystroke(stroke, cx);
        nanos(started.elapsed())
    });
    cx.background_executor()
        .timer(Duration::from_millis(30))
        .await;
    let painted: u64 = HIGHLIGHT_NS
        .load(Ordering::Relaxed)
        .saturating_sub(paint_before);
    Some(took.unwrap_or(0).saturating_add(painted))
}

fn report_keys(work: &mut [u64]) {
    let jobs: u64 = PARSE_JOBS.load(Ordering::Relaxed);
    eprintln!(
        "bb_keys sent={} ui_work_p50_us={} ui_work_p99_us={} ui_work_max_us={} parse_jobs={jobs} parse_mean_ms={}",
        work.len(),
        percentile(work, 50) / 1_000,
        percentile(work, 99) / 1_000,
        percentile(work, 100) / 1_000,
        PARSE_NS.load(Ordering::Relaxed) / jobs.max(1) / 1_000_000,
    );
}

pub(crate) fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

pub(crate) fn record_highlight(duration: Duration) {
    HIGHLIGHT_NS.fetch_add(nanos(duration), Ordering::Relaxed);
    HIGHLIGHT_CALLS.fetch_add(1, Ordering::Relaxed);
}

impl ExplorerView {
    pub(crate) fn record_keypress(&mut self, window: &mut Window) {
        if self.keypresses == 0 {
            self.latency_before = Some(window.input_latency_snapshot());
            gpui::profiler::set_trace_enabled(true);
            self.frame_collector = Some(gpui::profiler::FrameTimingCollector::new());
        }
        self.keypresses = self.keypresses.saturating_add(1);
        if self.keypresses == 500 {
            eprintln!("editor_stage keys=500 edit_total_us={} reparse_total_us={} highlight_total_us={} highlight_calls={}", self.edit_ns / 1_000, self.reparse_ns / 1_000, HIGHLIGHT_NS.load(Ordering::Relaxed) / 1_000, HIGHLIGHT_CALLS.load(Ordering::Relaxed));
            let collector: Option<gpui::profiler::FrameTimingCollector> =
                self.frame_collector.take();
            let Some(before): Option<gpui::profiler::InputLatencySnapshot> =
                self.latency_before.take()
            else {
                return;
            };
            window.on_next_frame(move |window: &mut Window, _: &mut gpui::App| {
                window.on_next_frame(move |window: &mut Window, _: &mut gpui::App| {
                    if let Some(mut collector) = collector {
                        frame_stages(collector.collect_unseen());
                    }
                    let frame: gpui::profiler::FrameDurationSnapshot = window.frame_duration_snapshot();
                    eprintln!("editor_hist draw_p50_us={} draw_p99_us={} dirty_present_p50_us={} dirty_present_p99_us={}", frame.draw_duration_histogram.value_at_quantile(0.50) / 1_000, frame.draw_duration_histogram.value_at_quantile(0.99) / 1_000, frame.dirty_to_present_histogram.value_at_quantile(0.50) / 1_000, frame.dirty_to_present_histogram.value_at_quantile(0.99) / 1_000);
                    gpui::profiler::set_trace_enabled(false);
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

fn frame_stages(events: Vec<gpui::profiler::FrameEvent>) {
    let mut draws: u64 = 0;
    let mut presents: u64 = 0;
    let mut layout: u64 = 0;
    let mut prepaint: u64 = 0;
    let mut paint: u64 = 0;
    let mut refresh: BTreeMap<u64, u64> = BTreeMap::new();
    let mut dirty_to_draw: Vec<u64> = Vec::new();
    let mut present_ns: u64 = 0;
    for event in events {
        match event {
            gpui::profiler::FrameEvent::Draw(frame) => {
                draws = draws.saturating_add(1);
                layout = layout.saturating_add(nanos(frame.phases.request_layout));
                prepaint = prepaint.saturating_add(nanos(frame.phases.prepaint));
                paint = paint.saturating_add(nanos(frame.phases.paint));
                if let Some(interval) = frame.refresh_interval {
                    *refresh.entry(nanos(interval) / 1_000).or_default() += 1;
                }
                if let Some(dirty) = frame.dirty_to_draw_duration() {
                    dirty_to_draw.push(nanos(dirty) / 1_000);
                }
            }
            gpui::profiler::FrameEvent::Present(frame) => {
                presents = presents.saturating_add(1);
                present_ns = present_ns.saturating_add(nanos(frame.present_duration()));
            }
        }
    }
    let divisor: u64 = draws.max(1).saturating_mul(1_000);
    eprintln!("editor_frame draws={draws} presents={presents} layout_mean_us={} prepaint_mean_us={} paint_mean_us={} present_mean_us={} dirty_draw_p50_us={} dirty_draw_p99_us={} refresh_us={refresh:?}", layout / divisor, prepaint / divisor, paint / divisor, present_ns / presents.max(1) / 1_000, percentile(&mut dirty_to_draw, 50), percentile(&mut dirty_to_draw, 99));
}

fn percentile(values: &mut [u64], quantile: usize) -> u64 {
    values.sort_unstable();
    let index: usize = values.len().saturating_sub(1).saturating_mul(quantile) / 100;
    values.get(index).copied().unwrap_or(0)
}
