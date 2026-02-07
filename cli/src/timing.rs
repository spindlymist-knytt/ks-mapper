use std::fmt::Display;
use std::io::Write;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Timespan {
    pub start: Instant,
    pub end: Option<Instant>,
}

pub fn time_it<F, T>(label: &str, f: F) -> T
where
    F: FnOnce() -> T
{
    print!("{label}");
    let _ = std::io::stdout().flush();
    let mut span = Timespan::begin();
    
    let result = f();
    
    span.end();
    println!(" [{span}]");
    
    result
}

pub fn time_it_anyhow<F, T>(label: &str, f: F) -> anyhow::Result<T>
where
    F: FnOnce() -> anyhow::Result<T>,
{
    time_it(label, f)
}

impl Timespan {
    pub fn begin() -> Self {
        Self {
            start: Instant::now(),
            end: None,
        }
    }
    
    pub fn end(&mut self) {
        self.end.replace(Instant::now());
    }
}

impl Display for Timespan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.end {
            Some(end) => {
                let duration = end - self.start;
                let secs = duration.as_secs_f64();
                if secs < 0.001 {
                    write!(f, "{:.1}us", secs * 1_000_000.0)
                }
                else if secs < 0.1 {
                    write!(f, "{:.1}ms", secs * 1000.0)
                }
                else {
                    write!(f, "{:.1}s", secs)
                }
            }
            None => {
                write!(f, "<timing>")
            }
        }
    }
}
