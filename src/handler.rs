use poem_openapi::OpenApi;
use crate::response::MyResponse; 
use crate::task_runner::TaskRunner;
use poem::web::Data;
use tracing::info;
use rand::Rng;
use std::sync::Arc;

pub(crate) enum HandlerError {
    Error
}

pub(crate) struct Api;

pub fn nth_prime(n: u32) -> Option<u64> {
    if n < 1 {
        return None;
    }

    // The prime counting function is pi(x) which is approximately x/ln(x)
    // A good upper bound for the nth prime is ceil(x * ln(x * ln(x)))
    let x = if n <= 10 { 10.0 } else { n as f64 };
    let limit: usize = (x * (x * (x).ln()).ln()).ceil() as usize;
    let mut sieve = vec![true; limit];
    let mut count = 0;

    // Exceptional case for 0 and 1
    sieve[0] = false;
    sieve[1] = false;

    for prime in 2..limit {
        if !sieve[prime] {
            continue;
        }
        count += 1;
        if count == n {
            return Some(prime as u64);
        }

        for multiple in ((prime * prime)..limit).step_by(prime) {
            sieve[multiple] = false;
        }
    }
    None
}

#[OpenApi]
impl Api {
    #[oai(path = "/prime", method = "post")]
    async fn read(&self, task_runner: Data<&Arc<TaskRunner>>) -> MyResponse<i32> {
        TaskRunner::run_task(*task_runner, async {
            let mut rng = rand::thread_rng();
            let limit = rng.gen_range(500_000..5_000_000);
            info!("Finding the {}th prime", limit);
            let now = std::time::Instant::now();
            if let Some(prime) = nth_prime(limit) {
                let elapsed = now.elapsed();
                info!("Found the {}th prime to be {} in {}ms", limit, prime, elapsed.as_millis());
            }
        });
        Ok(None).into()
    }
}