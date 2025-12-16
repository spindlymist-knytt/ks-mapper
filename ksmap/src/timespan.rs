use std::fmt::Display;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Timespan {
    pub start: Instant,
    pub end: Option<Instant>,
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
                    write!(f, "{:.1}us", secs / 1_000_000.0)
                }
                else if secs < 0.1 {
                    write!(f, "{:.1}ms", secs / 1000.0)
                }
                else {
                    write!(f, "{:.1}s", secs)
                }
            }
            None => {
                write!(f, "<running>")
            }
        }
    }
}
