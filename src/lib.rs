#![allow(unused)]

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::ops::{Deref, Range};
use std::pin::Pin;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};
use std::task::{Context, Poll};

// --- Tier 1 API --- //

/// Represents the logical time of the raft state machine.
pub type LogicalInstant = u64;

pub type ServerId = uuid::Uuid;

pub type TermIdx = u64;
pub type EntryIdx = u64;
pub type CommandInternalIdx = usize;

pub trait State {
    type Command: Clone; // TODO: Get rid of this restriction!

    fn apply(&mut self, cmd: Self::Command);
}

// There will be more fields in the future, so non_exhaustive
#[non_exhaustive]
pub struct RaftConfig {
    pub peers: Vec<ServerId>,
}

pub struct RaftStateMachine<S> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S> RaftStateMachine<S>
where
    S: State + Default,
{
    /// Creates a completely new raft state machine, with empty log.
    pub fn new(config: RaftConfig) -> Self {
        todo!()
    }
}

impl<S> RaftStateMachine<S>
where
    S: State,
{
    /// Asks the raft state machine to apply a command.
    pub fn apply_command(&mut self, command: S::Command) -> Result<(), ApplyCommandError> {
        todo!()
    }

    /// Advances the logical clock.
    pub fn on_tick(&mut self) {
        todo!()
    }

    /// Updates the state, based on a received message.
    pub fn on_message(&mut self, message: Message<S::Command>) {
        todo!()
    }

    /// Gets the state that has been committed so far.
    pub fn committed_state(&self) -> &S {
        todo!()
    }

    /// Polls for more raft commands to execute.
    pub fn poll_commands(&mut self, cx: &mut Context<'_>) -> Poll<RaftCommands<S::Command>> {
        todo!()
    }

    /// Returns the current role.
    pub fn role(&self) -> Role {
        todo!()
    }
}

pub enum Role {
    Follower,
    Candidate,
    Leader,
}

// There might be more fields in the future, so marking as non_exhaustive for now
#[non_exhaustive]
pub struct RaftCommands<Command> {
    /// Term number and server idx to store on disk.
    pub term_and_vote: Option<(TermIdx, ServerId)>,

    /// Log entries to persist.
    pub log_entries: Vec<LogEntry<Command>>,

    /// Messages to send. Must only be sent after the state is persisted
    /// to disk.
    pub messages: Vec<Message<Command>>,
}

pub struct LogEntry<Command> {
    pub term: TermIdx,
    pub index: EntryIdx,

    pub data: LogEntryData<Command>,
}

// Now, it can only be a "Command", but in the future we might add other variants
// Not marking as non-exhaustive yet, it would break too many things
pub enum LogEntryData<Command> {
    Command(Command),
}

pub enum Message<Command> {
    AppendEntriesRequest(AppendEntriesRequest<Command>),
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteRequest(RequestVoteRequest),
    RequestVoteRespose(RequestVoteResponse),
}

pub struct RequestVoteRequest {
    /// Target term of the request.
    pub term: u64,

    /// The UUID of the candidate that requests the vote.
    pub candidate_id: ServerId,

    /// The last entry index in the log before the election.
    pub last_log_entry: u64,

    /// The number of the last term before the election.
    pub last_log_term: u64,
}

pub struct RequestVoteResponse {
    /// True if vote has been granted. This will happen
    /// iff the candidate's vote is at least as up to date as the receiver.
    pub vote_granted: bool,

    /// Current term as perceived by the receiver.
    pub term: u64,
}

pub struct AppendEntriesRequest<Command> {
    /// Leader's term.
    pub term: u64,

    /// Information for the client so that they know
    /// where to redirect requests.
    pub leader_id: u64,

    /// Index of the log entry immediately preceding
    /// new ones.
    pub prev_log_index: u64,

    /// Term of the prev_log_index entry.
    pub prev_log_term: u64,

    /// Range of indexes of entries to store on the other side.
    pub entries: Range<LogEntry<Command>>, // TODO
}

pub struct AppendEntriesResponse {
    /// Current term, for the leader to update itself.
    pub term: u64,

    /// True if the follower contained entry matching
    /// prev_log_index and prev_log_term.
    pub success: bool,
}

#[derive(Debug)]
pub enum ApplyCommandError {
    NotLeader,
}

impl fmt::Display for ApplyCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLeader => write!(f, "cannot apply command, the current node is not a leader"),
        }
    }
}

impl Error for ApplyCommandError {}

// --- Tier 2 API --- //

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T>>>;

/// A user-facing object
pub struct RaftServer<S> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S> RaftServer<S>
where
    S: State,
{
    /// Creates a new server.
    ///
    /// Returns a pair: (server, a future that should be polled while the server exists)
    pub fn new(
        config: RaftConfig,
        my_id: ServerId,
        rpc: impl Rpc<S::Command>,
        persistence: impl Persistence<S::Command>,
    ) -> (RaftServer<S>, impl Future<Output = ()> + Send + 'static) {
        (todo!(), async {})
    }

    /// Asks the raft state machine to apply a command.
    ///
    /// Returns an object that allows to wait until the command is applied locally.
    pub fn apply_command(
        &mut self,
        command: S::Command,
    ) -> Result<impl Future<Output = Result<(), ApplyCommandError>>, ApplyCommandError> {
        Ok(async { todo!() })
    }

    /// Gets the currently applied state.
    #[allow(clippy::diverging_sub_expression)]
    pub fn local_state(&self) -> impl Deref<Target = &S> {
        let r: &S = todo!();
        &r
    }
}

pub trait Rpc<Command> {
    fn append_entries(
        &self,
        server_id: ServerId,
        req: AppendEntriesRequest<Command>,
    ) -> LocalBoxFuture<Result<AppendEntriesResponse, Box<dyn Error>>>;

    fn request_vote(
        &self,
        server_id: ServerId,
        req: RequestVoteRequest,
    ) -> LocalBoxFuture<Result<RequestVoteResponse, Box<dyn Error>>>;
}

pub trait Persistence<Command> {
    fn store_term_and_vote(
        &self,
        term: TermIdx,
        vote: ServerId,
    ) -> LocalBoxFuture<Result<(), Box<dyn Error>>>;

    fn load_term_and_vote(&self) -> LocalBoxFuture<Result<(TermIdx, ServerId), Box<dyn Error>>>;

    fn store_log_entries(
        &self,
        entries: &[LogEntry<Command>],
    ) -> LocalBoxFuture<Result<(), Box<dyn Error>>>;

    #[allow(clippy::type_complexity)]
    fn load_log(&self) -> LocalBoxFuture<Result<Vec<LogEntry<Command>>, Box<dyn Error>>>;
}
