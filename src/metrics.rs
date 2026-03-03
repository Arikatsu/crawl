use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct Metrics {
    pub total: AtomicUsize,
    pub http_errors: AtomicUsize,
    pub network_errors: AtomicUsize,
    pub logos_found: AtomicUsize,
}

impl Metrics {
    pub fn log_summary(&self) {
        let total = self.total.load(Ordering::Relaxed);
        let http_errs = self.http_errors.load(Ordering::Relaxed);
        let net_errs = self.network_errors.load(Ordering::Relaxed);
        let logos = self.logos_found.load(Ordering::Relaxed);

        let reachable = total.saturating_sub(http_errs + net_errs);
        let true_hit_rate = if reachable > 0 { (logos as f64 / reachable as f64) * 100.0 } else { 0.0 };

        eprintln!("\n --- INTERNAL CRAWL METRICS ---");
        eprintln!("Total Domains Processed: {}", total);
        eprintln!("- HTTP Errors:       {}", http_errs);
        eprintln!("- Network Errors:    {}", net_errs);
        eprintln!("- Reachable Domains: {}", reachable);
        eprintln!("--------------------------------");
        eprintln!("Logos Found:         {} (True Hit Rate: {:.1}% of reachable HTML)", logos, true_hit_rate);
        eprintln!("--------------------------------\n");
    }
}