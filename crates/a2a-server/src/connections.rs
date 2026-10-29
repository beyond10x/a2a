//! The connections: how many the server holds, how long it waits for a request, and the writer
//! thread of each, so that no client's socket holds back another client.
//!
//! One thread accepts. Each connection has a thread that reads its request and then reads on
//! until the client hangs up, and, once the request is read, a thread that writes what the router
//! queues for it. The router only ever queues, and never waits on a socket.

use std::io::{BufReader, ErrorKind, Read as _, Write as _};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::time::{Duration, Instant};

use a2a::server::http;

use crate::Limits;
use crate::router;

/// How many bytes wait for one connection's writer at most. A stream whose client reads more
/// slowly than its task's events come fills it and is dropped; a client that keeps up is not
/// dropped for a burst of many small events.
const BACKLOG: usize = 4 * 1024 * 1024;
/// File descriptors kept free of connections: standard streams, the listener, and the
/// connections being closed.
const DESCRIPTOR_RESERVE: usize = 16;
/// File descriptors a connection holds: the reader's and the writer's (or, before and after its
/// answer, the one the server ends it with).
const DESCRIPTORS_PER_CONNECTION: usize = 2;
/// How often the server looks for connections past their deadline.
const WATCH: Duration = Duration::from_millis(100);
/// How long accepting pauses after it fails, for instance with every file descriptor in use.
const ACCEPT_PAUSE: Duration = Duration::from_millis(100);

/// A connection whose request is being read, or was read.
const READING: u8 = 0;
/// A connection whose request did not come in before the deadline.
const EXPIRED: u8 = 1;
/// A connection closed to seat another.
const EVICTED: u8 = 2;

/// What a connection's writer writes.
pub(crate) enum Outgoing {
    /// One whole answer.
    Answer(http::Response),
    /// Bytes of an event stream: its head and events.
    Text(Arc<str>),
}

impl Outgoing {
    /// The bytes it holds, as the backlog counts them.
    fn size(&self) -> usize {
        match self {
            Self::Answer(answer) => answer.body.len(),
            Self::Text(text) => text.len(),
        }
    }
}

/// A connection as the router holds it: the queue its writer drains, the bytes in that queue not
/// yet written, and whether its client hung up.
pub(crate) struct Peer {
    queue: Sender<Outgoing>,
    queued: Arc<AtomicUsize>,
    gone: Arc<AtomicBool>,
    cut: Arc<AtomicBool>,
}

impl Peer {
    /// Queues `outgoing` without waiting; `false` where it would take the bytes waiting for the
    /// writer past [`BACKLOG`], or the writer stopped. Where nothing waits it is always queued, so
    /// an answer larger than the backlog is still written.
    pub(crate) fn offer(&self, outgoing: Outgoing) -> bool {
        let size = outgoing.size();
        let before = self.queued.fetch_add(size, Ordering::AcqRel);
        if before > 0 && before.saturating_add(size) > BACKLOG {
            self.queued.fetch_sub(size, Ordering::AcqRel);
            return false;
        }
        if self.queue.send(outgoing).is_err() {
            self.queued.fetch_sub(size, Ordering::AcqRel);
            return false;
        }
        true
    }

    /// Whether the client's side ended: its socket read end of file or failed.
    pub(crate) fn gone(&self) -> bool {
        self.gone.load(Ordering::Acquire)
    }

    /// Closes the connection without writing what is still queued.
    pub(crate) fn cut(&self) {
        self.cut.store(true, Ordering::Release);
    }
}

/// Where a connection is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Its request head is awaited.
    Head,
    /// Its request is in and its writer holds the socket.
    Answering,
    /// Its answer is written; what the client still sends is read and dropped.
    Answered,
}

/// A connection the server holds.
struct Held {
    id: u64,
    /// When its deadline started: when it opened, or when its answer was written. Bytes arriving
    /// do not move it.
    since: Instant,
    phase: Phase,
    /// A handle on the socket to end it from outside, at its deadline or to seat another
    /// connection: there while its request head is awaited and once its answer is written, and
    /// held by its writer in between.
    handle: Option<TcpStream>,
    fate: Arc<AtomicU8>,
}

#[derive(Default)]
struct Book {
    held: Vec<Held>,
    next: u64,
}

fn lock(book: &Mutex<Book>) -> MutexGuard<'_, Book> {
    book.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A connection's place in the book, given back when the last of its threads ends.
struct Seat {
    id: u64,
    book: Arc<Mutex<Book>>,
}

impl Drop for Seat {
    fn drop(&mut self) {
        lock(&self.book).held.retain(|held| held.id != self.id);
    }
}

/// Starts accepting on `listener`, and hands each request read to `requests` with its [`Peer`].
pub(crate) fn start(
    listener: TcpListener,
    requests: Sender<(http::Request, Peer)>,
    limits: Limits,
) -> std::io::Result<()> {
    let book = Arc::new(Mutex::new(Book::default()));
    let watched = Arc::downgrade(&book);
    let timeout = limits.request_timeout;
    std::thread::Builder::new()
        .name("a2a-watch".to_owned())
        .spawn(move || watch(&watched, timeout))?;
    let capacity = capacity(limits.max_connections, descriptor_limit());
    std::thread::Builder::new()
        .name("a2a-accept".to_owned())
        .spawn(move || accept(&listener, &requests, &book, capacity, timeout))?;
    Ok(())
}

/// How many connections the server holds: `asked`, or fewer where the file descriptor limit
/// would run out first, so that a connection over the cap is closed rather than left unaccepted.
fn capacity(asked: usize, descriptors: Option<usize>) -> usize {
    descriptors.map_or(asked, |limit| {
        asked
            .min(limit.saturating_sub(DESCRIPTOR_RESERVE) / DESCRIPTORS_PER_CONNECTION)
            .max(1)
    })
}

/// The soft limit on open file descriptors, where `/proc/self/limits` says it.
fn descriptor_limit() -> Option<usize> {
    let limits = std::fs::read_to_string("/proc/self/limits").ok()?;
    limits
        .lines()
        .find_map(|line| line.strip_prefix("Max open files"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// Accepts connections for good, each read on a thread of its own.
fn accept(
    listener: &TcpListener,
    requests: &Sender<(http::Request, Peer)>,
    book: &Arc<Mutex<Book>>,
    capacity: usize,
    timeout: Duration,
) {
    for connection in listener.incoming() {
        let stream = match connection {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("a2a-server: accepting a connection: {error}");
                std::thread::sleep(ACCEPT_PAUSE);
                continue;
            }
        };
        let Some((seat, fate)) = admit(book, &stream, capacity) else {
            continue;
        };
        let requests = requests.clone();
        let started = std::thread::Builder::new()
            .name("a2a-connection".to_owned())
            .spawn(move || read(stream, &requests, seat, &fate, timeout));
        if let Err(error) = started {
            eprintln!("a2a-server: starting a connection's thread: {error}");
        }
    }
}

/// A seat for `stream`, or `None`, which closes it. At `capacity`, the connection waiting longest
/// for a request head, before its answer or after it, is closed to seat this one, so that clients
/// sending slowly, or sending on after their answer, cannot lock another out; where every
/// connection held is being answered, this one is refused.
fn admit(
    book: &Arc<Mutex<Book>>,
    stream: &TcpStream,
    capacity: usize,
) -> Option<(Arc<Seat>, Arc<AtomicU8>)> {
    let handle = stream.try_clone().ok()?;
    let mut seated = lock(book);
    if seated.held.len() >= capacity {
        let oldest = seated
            .held
            .iter()
            .enumerate()
            .filter(|(_, held)| held.handle.is_some())
            .min_by_key(|(_, held)| held.since)
            .map(|(at, _)| at)?;
        let evicted = seated.held.swap_remove(oldest);
        evicted.fate.store(EVICTED, Ordering::Release);
        if let Some(handle) = &evicted.handle {
            let _ = handle.shutdown(Shutdown::Both);
        }
    }
    let id = seated.next;
    seated.next += 1;
    let fate = Arc::new(AtomicU8::new(READING));
    seated.held.push(Held {
        id,
        since: Instant::now(),
        phase: Phase::Head,
        handle: Some(handle),
        fate: Arc::clone(&fate),
    });
    drop(seated);
    let seat = Arc::new(Seat {
        id,
        book: Arc::clone(book),
    });
    Some((seat, fate))
}

/// Ends every connection whose deadline passed, until the server is gone: one still sending its
/// request head is answered 408 by its reader, and one whose answer is written is closed.
// ESS gap (ess 0.67.0): the generated `http::read` bounds each read, not the request, and the
// generated layer holds no connections, so the request deadline and the cap are kept here.
fn watch(book: &Weak<Mutex<Book>>, timeout: Duration) {
    while let Some(book) = book.upgrade() {
        let now = Instant::now();
        for held in &mut lock(&book).held {
            if now.duration_since(held.since) < timeout {
                continue;
            }
            let Some(handle) = held.handle.take() else {
                continue;
            };
            if held.phase == Phase::Head {
                held.fate.store(EXPIRED, Ordering::Release);
                let _ = handle.shutdown(Shutdown::Read);
            } else {
                let _ = handle.shutdown(Shutdown::Both);
            }
        }
        drop(book);
        std::thread::sleep(WATCH);
    }
}

/// The writer's handle on the connection, taken out of the book; `None` where the connection was
/// ended while its request was read.
fn taken(seat: &Seat) -> Option<TcpStream> {
    let mut book = lock(&seat.book);
    let held = book.held.iter_mut().find(|held| held.id == seat.id)?;
    let handle = held.handle.take()?;
    held.phase = Phase::Answering;
    Some(handle)
}

/// Gives the connection's handle back once its answer is written: its deadline starts again now,
/// and it may be closed to seat another connection.
fn answered(seat: &Seat, handle: TcpStream) {
    if let Some(held) = lock(&seat.book)
        .held
        .iter_mut()
        .find(|held| held.id == seat.id)
    {
        held.phase = Phase::Answered;
        held.since = Instant::now();
        held.handle = Some(handle);
    }
}

/// Reads one request off `stream`, hands it to the server with a writer for its answer, then
/// reads on until the client hangs up. A request that does not read is refused here; a
/// connection on which nothing arrives before it is ended is closed without an answer.
fn read(
    stream: TcpStream,
    requests: &Sender<(http::Request, Peer)>,
    seat: Arc<Seat>,
    fate: &AtomicU8,
    timeout: Duration,
) {
    let _ = stream.set_read_timeout(Some(http::READ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(http::WRITE_TIMEOUT));
    if !arrives(&stream, fate) {
        return;
    }
    let mut reader = BufReader::new(stream);
    let read = http::read(&mut reader);
    let writer = taken(&seat);
    let mut stream = reader.into_inner();
    let (request, writer) = match (read, writer) {
        (Ok(request), Some(writer)) => (request, writer),
        (Err(refusal), Some(writer)) => {
            let _ = http::write(&mut stream, &router::refused_answer(&refusal));
            answered(&seat, writer);
            http::linger(&mut stream);
            return;
        }
        (_, None) => {
            if fate.load(Ordering::Acquire) == EXPIRED {
                refuse(&mut stream, &router::late_answer(timeout));
            }
            return;
        }
    };
    let gone = Arc::new(AtomicBool::new(false));
    let cut = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    let queued = Arc::new(AtomicUsize::new(0));
    let (queue, outgoing) = mpsc::channel();
    let started = {
        let (cut, done, queued, seat) = (
            Arc::clone(&cut),
            Arc::clone(&done),
            Arc::clone(&queued),
            Arc::clone(&seat),
        );
        std::thread::Builder::new()
            .name("a2a-write".to_owned())
            .spawn(move || write(writer, outgoing, &queued, &cut, &done, seat))
    };
    if let Err(error) = started {
        eprintln!("a2a-server: starting a connection's writer: {error}");
        return;
    }
    let peer = Peer {
        queue,
        queued,
        gone: Arc::clone(&gone),
        cut,
    };
    if requests.send((request, peer)).is_err() {
        return;
    }
    listen(&stream, &done);
    gone.store(true, Ordering::Release);
}

/// Writes an answer to a request that was not read, and closes the connection.
fn refuse(stream: &mut TcpStream, answer: &http::Response) {
    let _ = http::write(stream, answer);
    http::linger(stream);
}

/// Waits for the first byte of a request without taking it; `false` where the connection closes,
/// or is ended at its deadline or to seat another, before any arrives.
fn arrives(stream: &TcpStream, fate: &AtomicU8) -> bool {
    let mut first = [0_u8; 1];
    loop {
        match stream.peek(&mut first) {
            Ok(0) => return false,
            Ok(_) => return true,
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) =>
            {
                if fate.load(Ordering::Acquire) != READING {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }
}

/// Reads what the client sends after its request, and returns once it hangs up: its socket reads
/// end of file or fails, its answer is written in full and nothing comes for a read timeout, or
/// the server ends it at its deadline or to seat another. Reading, not peeking, so that a client
/// that sends bytes and hangs up is seen to go.
fn listen(mut stream: &TcpStream, done: &AtomicBool) {
    let mut sink = [0_u8; 4096];
    loop {
        match stream.read(&mut sink) {
            Ok(0) => return,
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) =>
            {
                if done.load(Ordering::Acquire) {
                    return;
                }
            }
            Err(_) => return,
        }
    }
}

/// Writes what the router queues for one connection until it queues no more, then closes the
/// connection's sending side and gives its handle back to the book. A write that fails, or a
/// [`Peer::cut`], closes it whole.
fn write(
    mut stream: TcpStream,
    outgoing: Receiver<Outgoing>,
    queued: &AtomicUsize,
    cut: &AtomicBool,
    done: &AtomicBool,
    seat: Arc<Seat>,
) {
    let mut whole = false;
    for item in &outgoing {
        if cut.load(Ordering::Acquire) {
            whole = true;
            break;
        }
        let size = item.size();
        let written = match item {
            Outgoing::Answer(answer) => http::write(&mut stream, &answer),
            Outgoing::Text(text) => stream
                .write_all(text.as_bytes())
                .and_then(|()| stream.flush()),
        };
        queued.fetch_sub(size, Ordering::AcqRel);
        if written.is_err() {
            whole = true;
            break;
        }
    }
    drop(outgoing);
    if whole || cut.load(Ordering::Acquire) {
        let _ = stream.shutdown(Shutdown::Both);
    } else {
        let _ = stream.shutdown(Shutdown::Write);
        answered(&seat, stream);
    }
    done.store(true, Ordering::Release);
    drop(seat);
}

#[cfg(test)]
mod tests {
    use super::capacity;

    #[test]
    fn the_cap_leaves_room_under_the_descriptor_limit() {
        assert_eq!(capacity(256, None), 256);
        assert_eq!(capacity(256, Some(1024)), 256);
        assert_eq!(capacity(256, Some(64)), 24);
        assert_eq!(capacity(10, Some(64)), 10);
        assert_eq!(capacity(256, Some(4)), 1);
    }
}
