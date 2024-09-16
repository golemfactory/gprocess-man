use anyhow::{anyhow, bail, Result};
// use tokio::sync::Mutex;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use tokio::sync::Mutex;
// use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::Arc;
use tokio::io::AsyncReadExt;

pub type Pid = u64;

pub fn create() -> ProcessManager {
    let inner = Default::default();
    ProcessManager { inner }
}

#[derive(Clone)]
pub struct ProcessManager {
    inner: Arc<Mutex<HashMap<Pid, Arc<ChildInfo>>>>,
}

impl ProcessManager {
    pub async fn add_process(&self, mut child: Child) -> Result<Pid> {
        let stdin = child.stdin.take().map(Into::into);
        let stdout = child.stdout.take().map(Into::into);
        let stderr = child.stderr.take().map(Into::into);
        let pid = match child.id() {
            Some(pid) => pid as Pid,
            None => bail!("failed to get pid"),
        };
        let ci = Arc::new(ChildInfo {
            child: Mutex::new(child),
            stdin,
            stdout,
            stderr,
        });
        let mut g = self.inner.lock().await;
        g.insert(pid, ci);
        Ok(pid)
    }

    pub async fn get_reader(&self, pid: Pid, fd: i32) -> Result<ReadHandle> {
        let pi = self.pi(pid).await?;

        let h = match fd {
            1 => pi
                .stdout
                .clone()
                .ok_or_else(|| anyhow!("stdout is not piped"))?,
            2 => pi
                .stderr
                .clone()
                .ok_or_else(|| anyhow!("stdout is not piped"))?,
            _ => bail!("invalid fd {fd}"),
        };

        Ok(h)
    }

    pub async fn get_writer(&self, pid: Pid, fd: i32) -> Result<WriteHandle> {
        let pi = self.pi(pid).await?;

        let h = match fd {
            0 => pi.stdin.clone().ok_or_else(|| anyhow!("stdin is not piped"))?,
            _ => bail!("invalid fd {}", fd),
        };

        Ok(h)
    }

    pub async fn wait(&self, pid: Pid) -> Result<i32> {
        let mut pi = self.pi(pid).await?;
        let status = pi.child.lock().await.wait().await?.code().unwrap_or(-1);
        self.inner.lock().await.remove(&pid);
        Ok(status)
    }

    pub async fn process_exists(&self, pid: Pid) -> bool {
        let mut g = self.inner.lock().await;
        g.get(&pid).is_some()
    }

    pub async fn ps(&self) -> Vec<Pid> {
        let mut g = self.inner.lock().await;
        g.keys().cloned().collect()
    }

    async fn pi(&self, pid: Pid) -> anyhow::Result<Arc<ChildInfo>> {
        let mut g = self.inner.lock().await;
        let pi = g.get(&pid).ok_or_else(|| anyhow!("pid {pid} not found"))?;
        Ok(Arc::clone(pi))
    }
}

struct ChildInfo {
    child: Mutex<Child>,
    stdin: Option<WriteHandle>,
    stdout: Option<ReadHandle>,
    stderr: Option<ReadHandle>,
}

#[derive(Clone)]
pub struct ReadHandle {
    inner: Arc<Mutex<Box<dyn AsyncRead + Send + Unpin>>>,
}

impl From<ChildStdout> for ReadHandle {
    fn from(value: ChildStdout) -> Self {
        let r: Box<dyn AsyncRead + Send + Unpin> = Box::new(value);
        let inner = Arc::new(Mutex::new(r));
        Self { inner }
    }
}

impl From<ChildStderr> for ReadHandle {
    fn from(value: ChildStderr) -> Self {
        let r: Box<dyn AsyncRead + Send + Unpin> = Box::new(value);
        let inner = Arc::new(Mutex::new(r));
        Self { inner }
    }
}

impl ReadHandle {
    pub async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut g = self
            .inner
            .try_lock()
            .map_err(|_| std::io::Error::other(anyhow!("concurrent read")))?;
        g.read(buf).await
    }
}

#[derive(Clone)]
pub struct WriteHandle {
    inner: Arc<Mutex<Box<dyn AsyncWrite + Send + Unpin>>>,
}

impl From<ChildStdin> for WriteHandle {
    fn from(value: ChildStdin) -> Self {
        let w: Box<dyn AsyncWrite + Send + Unpin> = Box::new(value);
        let inner = Arc::new(Mutex::new(w));
        Self { inner }
    }
}

impl WriteHandle {
    pub async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut g = self
            .inner
            .try_lock()
            .map_err(|_| std::io::Error::other(anyhow!("concurrent write")))?;
        g.write(buf).await
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        let mut g = self.inner.lock().await;
        g.flush().await
    }

    // fn poll_write(
    //     self: std::pin::Pin<&mut Self>,
    //     cx: &mut std::task::Context<'_>,
    //     buf: &[u8],
    // ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
    //     let mut g = self
    //         .inner
    //         .try_lock()
    //         .ok_or_else(|| std::io::Error::other(anyhow!("concurrent write")))?;
    //     g.poll_write(cx, buf)
    //     // std::pin::Pin::new(&mut g).write(buf)
    // }

    // fn poll_flush(
    //     self: std::pin::Pin<&mut Self>,
    //     cx: &mut std::task::Context<'_>,
    // ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
    //     todo!()
    // }

    // fn poll_shutdown(
    //     self: std::pin::Pin<&mut Self>,
    //     cx: &mut std::task::Context<'_>,
    // ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
    //     todo!()
    // }
}
