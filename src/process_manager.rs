use std::collections::HashMap;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::Arc;
use anyhow::{anyhow, bail, Result};
use parking_lot::Mutex;

pub type Pid = u64;

pub fn create() -> ProcessManager {
    todo!()
}

#[derive(Clone)]
pub struct ProcessManager {
    inner : Arc<Mutex<HashMap<Pid, Arc<ChildInfo>>>>
}

impl ProcessManager {

    pub fn add_process(&self, mut child : Child) -> Result<Pid> {

        let stdin = child.stdin.take().map(Into::into);
        let stdout = child.stdout.take().map(Into::into);
        let stderr = child.stderr.take().map(Into::into);
        let pid = child.id() as Pid;
        let ci = Arc::new(ChildInfo {
            child,
            stdin,
            stdout,
            stderr
        });
        let mut g = self.inner.lock();
        g.insert(pid, ci);
        Ok(pid)
    }

    pub fn get_reader(&self, pid : Pid, fd : i32) -> Result<ReadHandle> {
        let pi = self.pi(pid)?;

        let h = match fd {
            1 => pi.stdout.clone().ok_or_else(|| anyhow!("stdout is not piped"))?,
            2 => pi.stderr.clone().ok_or_else(|| anyhow!("stdout is not piped"))?,
            _ => bail!("invalid fd {fd}")
        };

        Ok(h)
    }

    pub fn get_writer(&self, pid : Pid, fd : i32) -> Result<WriteHandle> {
        todo!()
    }

    fn pi(&self, pid : Pid) -> anyhow::Result<Arc<ChildInfo>> {
        let mut g = self.inner.lock();
        let pi = g.get(&pid).ok_or_else(|| anyhow!("pid {pid} not found"))?;
        Ok(Arc::clone(pi))
    }
}

struct ChildInfo {
    child: Child,
    stdin: Option<WriteHandle>,
    stdout: Option<ReadHandle>,
    stderr: Option<ReadHandle>,
}

#[derive(Clone)]
pub struct ReadHandle {
    inner : Arc<Mutex<Box<dyn Read + Send>>>
}

impl From<ChildStdout> for ReadHandle {
    fn from(value: ChildStdout) -> Self {
        let r : Box<dyn Read + Send> = Box::new(value);
        let inner = Arc::new(Mutex::new(r));
        Self {
            inner
        }
    }
}

impl From<ChildStderr> for ReadHandle {
    fn from(value: ChildStderr) -> Self {
        let r : Box<dyn Read + Send> = Box::new(value);
        let inner = Arc::new(Mutex::new(r));
        Self {
            inner
        }
    }
}


impl Read for ReadHandle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut g= self.inner.try_lock().ok_or_else(|| std::io::Error::other(anyhow!("concurrent read")))?;
        g.read(buf)
    }
}

#[derive(Clone)]
pub struct WriteHandle {
    inner : Arc<Mutex<Box<dyn Write + Send>>>
}

impl From<ChildStdin> for WriteHandle {
    fn from(value: ChildStdin) -> Self {
        let w : Box<dyn Write + Send> = Box::new(value);
        let inner = Arc::new(Mutex::new(w));
        Self {
            inner
        }
    }
}



impl Write for WriteHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut g= self.inner.try_lock().ok_or_else(|| std::io::Error::other(anyhow!("concurrent write")))?;
        g.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut g= self.inner.lock();
        g.flush()
    }
}