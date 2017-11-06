use std::env;
use std::io::prelude::*;
use std::io;
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
    let t = thread::spawn(move || copy(start, &mut stderr, &mut io::stderr()));
    copy(start, &mut stdout, &mut io::stdout());
    t.join().unwrap();


    let status = child.wait().expect("failed to wait on child");
    if !status.success() {
        match status.code() {
            Some(i) => process::exit(i),
            None => panic!("subprocess failed with: {}", status),
        }
    }
}

fn copy(start: Instant, input: &mut Read, output: &mut Write) {
    match do_copy(&start, input, output) {
        Ok(()) => {}
        Err(err) => {
            drop(write!(output, "failed to read/write stream: {}\n", err).and_then(|()| {
                output.flush()
            }));
            process::exit(23);
        }
    }
}

fn do_copy(start: &Instant,
           input: &mut Read,
           output: &mut Write) -> io::Result<()> {
    let mut buf = [0; 1024];
    let mut saw_newline = true;
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            return Ok(())
        }

        let mut buf = &buf[..n];
        while buf.len() > 0 {
            if saw_newline {
                let dur = start.elapsed().as_secs();
                write!(output,
                       "[{:02}:{:02}:{:02}] ",
                       dur / 3600,
                       (dur % 3600) / 60,
                       dur % 60)?;
                saw_newline = false;
            }
            match buf.iter().position(|b| *b == b'\n') {
                Some(i) => {
                    output.write_all(&buf[..i + 1])?;
                    buf = &buf[i + 1..];
                    saw_newline = true;
                }
                None => {
                    output.write_all(buf)?;
                    break
                }
            }
        }
        output.flush()?;
    }
}
