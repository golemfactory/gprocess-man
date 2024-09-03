use anyhow::bail;
use nix::libc::{tcgetattr, termios};
use nix::pty::{forkpty, ForkptyResult, Winsize};
use nix::sys::termios::Termios;
use nix::unistd::{execv, Pid};
use std::ffi::CStr;
use std::os::fd::{AsRawFd, OwnedFd};
use std::{mem, thread};

unsafe fn run_cmd(cmd: &[u8]) -> anyhow::Result<(Pid, OwnedFd)> {
    let mut tc: termios = mem::zeroed();

    if tcgetattr(0, &mut tc as *mut termios) == -1 {
        bail!("failed to get term cfg");
    };
    let cmd = cmd.as_ref();
    let cmd = CStr::from_bytes_until_nul(cmd.as_ref())?;
    let tc = Termios::from(tc);
    match forkpty(
        Some(&Winsize {
            ws_row: 25,
            ws_col: 80,
            ws_xpixel: 25 * 16,
            ws_ypixel: 80 * 8,
        }),
        Some(&tc),
    )? {
        ForkptyResult::Child => {
            execv(cmd, &[cmd])?;
            panic!("unexpected")
        }
        ForkptyResult::Parent { child, master } => {
            //let fd = master.as_raw_fd();
            //ioctl_write_int!(fd, nix::libc::TIOCPKT, 1);
            Ok((child, master))
        }
    }
}

fn main() -> anyhow::Result<()> {
    unsafe {
        let (pid, master) = run_cmd("/opt/homebrew/bin/mc\0".as_bytes())?;

        crossterm::terminal::enable_raw_mode()?;

        {
            let master_fd = master.as_raw_fd();
            thread::spawn(move || {
                let mut buf = [0u8; 1000];
                let stdout = std::io::stdout();
                loop {
                    let n = nix::unistd::read(master_fd, &mut buf[..])?;
                    if n <= 0 {
                        break;
                    }
                    nix::unistd::write(&stdout, &buf[..n])?;
                }
                anyhow::Ok(())
            });
        }
        {
            let _ = thread::spawn(move || {
                let mut buf = [0u8; 1000];
                let stdin = std::io::stdin();
                loop {
                    let n = nix::unistd::read(stdin.as_raw_fd(), &mut buf[..])?;

                    if n <= 0 {
                        break;
                    }

                    let nr = nix::unistd::write(&master, &buf[..n])?;
                    if nr != n {
                        break;
                    }
                }
                anyhow::Ok(())
            });

            if let Err(e) = nix::sys::wait::waitpid(pid, None) {
                eprintln!("err: {:?}", e);
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    eprintln!("!!!done!!!");
    Ok(())
}
