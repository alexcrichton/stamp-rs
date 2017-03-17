use std::env;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::Instant;

fn main() {
    let mut args = env::args_os().skip(1);
    let mut cmd = match args.next() {
        Some(arg) => Command::new(arg),
        None => return,
    };
    for arg in args {
        cmd.arg(arg);
    }

    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let start = Instant::now();
    let mut child = cmd.spawn().expect("failed to spawn child process");
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let t = thread::spawn(move || copy(start, &mut stdout, &mut io::stdout()));
    copy(start, &mut stderr, &mut io::stderr());
    t.join().unwrap();


    let status = child.wait().expect("failed to wait on child");
    if !status.success() {
        match status.code() {
            Some(i) => process::exit(i),
            None => panic!("subprocess failed with: {}", status),
        }
    }
}

fn copy(start: Instant,
        input: &mut Read,
        output: &mut Write) {
    let input = BufReader::new(input);
    let mut output = BufWriter::new(output);
    for line in input.lines() {
        let dur = start.elapsed().as_secs();

        let result = line.and_then(|line| {
            write!(output,
                   "[{:02}:{:02}:{:02}] {}\n",
                   dur / 3600,
                   (dur % 3600) / 60,
                   dur % 60,
                   line)
        }).and_then(|()| {
            output.flush()
        });

        let err = match result {
            Ok(()) => continue,
            Err(e) => e,
        };
        drop(write!(output, "failed to read stream: {}\n", err).and_then(|()| {
            output.flush()
        }));
        process::exit(23);
    }
}
