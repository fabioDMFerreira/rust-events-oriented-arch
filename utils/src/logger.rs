use std::fs::File;
use std::io::Write;

use chrono::Local;
use env_logger::Env;

pub fn init_logger(logs_path: String) {
    let mut target = env_logger::Target::Stdout;

    if !logs_path.is_empty() {
        let file = Box::new(File::create("/var/log/app/stdout.log").expect("Can't create file"));

        target = env_logger::Target::Pipe(file);
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(target)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} {}:{} {}",
                Local::now().format("%b %d %H:%M:%S"),
                record.level(),
                record.file_static().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
}
