//! Fakes of the ports for testing the modes, driven through a scenario.
//!
//! A scenario sets up the state the mode starts in and names the mode to run. What follows is a
//! timeline: the events the world throws at the mode at moments of virtual time, and between them
//! the effects the mode is expected to have on the world. The chain is a declaration; its last
//! call runs the mode. The fakes play the events back as the mode calls into the ports and record
//! what actually happened in one trace, which must match the timeline entry for entry once the
//! mode returns. Time passes only while the keyboard waits for a key, by exactly the timeout the
//! mode asked for, so a test runs in no wall time at all.
//!
//! The fakes check the port protocol on every call and panic at the call that breaks it: a timed
//! keyboard read without a poll since the previous read, a blocking read while a command runs, a
//! start while a command runs, or a command dropped while still running.

use crate::config::ApplicationConfig;
use crate::logic::Context;
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::port::WindowManager;
use crate::port::clock::Clock;
use crate::port::executor::{CommandPoll, Executor, ExecutorError, RunningCommand};
use crate::port::input::Keyboard;
use crate::port::view::{View, ViewAction, ViewModel};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Something the world does to the mode at a moment.
///
/// A fault holds from its moment on until an event lifts it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    /// The user presses the key. It waits in the keyboard until the mode reads it.
    KeyPressed(Key),
    /// The running command finishes with the result, which the next poll reports.
    CommandFinishes(Result<String, ExecutorError>),
    /// Every start from now on fails.
    StartsFail,
    /// Starts work again.
    StartsSucceed,
    /// Every kill from now on fails, leaving the command in an unknown state.
    KillsFail,
}

/// Something the mode did to the world through a port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    Started(String),
    Background(String),
    Rendered(Screen),
    Killed,
}

/// What a rendered view model showed, reduced to what the tests assert on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Screen {
    Running {
        command_name: String,
        elapsed_secs: u64,
    },
    Error {
        error_type: String,
        retry_offered: bool,
    },
    LayerNavigation,
    ParameterInput {
        parameter_name: String,
    },
    None,
}

impl From<&ViewModel<'_>> for Screen {
    fn from(model: &ViewModel<'_>) -> Self {
        match model {
            ViewModel::None => Screen::None,
            ViewModel::CommandRunning(model) => Screen::Running {
                command_name: model.command_name.to_string(),
                elapsed_secs: model.elapsed.as_secs(),
            },
            ViewModel::Error(model) => Screen::Error {
                error_type: model.error_type.to_string(),
                retry_offered: model
                    .actions
                    .iter()
                    .any(|(_, action)| matches!(action, ViewAction::Retry)),
            },
            ViewModel::LayerNavigation(_) => Screen::LayerNavigation,
            ViewModel::ParameterInput(model) => Screen::ParameterInput {
                parameter_name: model.parameter_name.to_string(),
            },
        }
    }
}

/// The world before the mode runs.
///
/// `BackSpace` is the back key, Escape the exit key and Return the retry key.
#[must_use]
pub(crate) struct Scenario {
    given: Vec<Event>,
}

impl Scenario {
    pub(crate) fn new() -> Self {
        Self { given: vec![] }
    }

    /// The event has happened by the time the mode starts.
    pub(crate) fn given(mut self, event: Event) -> Self {
        self.given.push(event);
        self
    }

    /// The mode runs from here on. What follows is its timeline.
    pub(crate) fn run<F, T>(self, mode: F) -> Timeline<F>
    where
        F: FnOnce(&Context<'_>) -> T,
    {
        Timeline {
            given: self.given,
            entries: vec![],
            mode: Some(mode),
        }
    }
}

/// One entry of a timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    At(Duration, Event),
    Effect(Effect),
}

impl Entry {
    /// The entry as the timeline states it.
    fn describe(&self) -> String {
        match self {
            Entry::At(at, event) => format!("at {} ms: {event:?}", at.as_millis()),
            Entry::Effect(effect) => format!("{effect:?}"),
        }
    }

    /// The entry without its moment.
    fn content(&self) -> String {
        match self {
            Entry::At(_, event) => format!("{event:?}"),
            Entry::Effect(effect) => format!("{effect:?}"),
        }
    }
}

/// What happens while the mode runs: scripted events at moments counted from the start of the
/// run, and the effects expected between them. The timeline is told by its last call, `returns`.
#[must_use]
pub(crate) struct Timeline<F> {
    given: Vec<Event>,
    entries: Vec<Entry>,
    /// The mode to run, until it has been run.
    mode: Option<F>,
}

impl<F, T> Timeline<F>
where
    F: FnOnce(&Context<'_>) -> T,
    T: PartialEq + Debug,
{
    /// The event happens at the moment, in milliseconds since the run began. Moments must not
    /// decrease.
    pub(crate) fn at(mut self, ms: u64, event: Event) -> Self {
        let at = Duration::from_millis(ms);
        let last = self.entries.iter().rev().find_map(|entry| match entry {
            Entry::At(at, _) => Some(*at),
            Entry::Effect(_) => None,
        });
        assert!(
            last.is_none_or(|last| last <= at),
            "the timeline goes back in time at {ms} ms"
        );
        self.entries.push(Entry::At(at, event));
        self
    }

    /// The mode is expected to have the effect before the next event happens.
    pub(crate) fn effect(mut self, effect: Effect) -> Self {
        self.entries.push(Entry::Effect(effect));
        self
    }

    /// The mode returns the expected result at the end of the timeline.
    ///
    /// Runs the mode, checks that the run followed the timeline and that the mode returned the
    /// expected result.
    #[track_caller]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "the expected result is the last entry of the timeline and is handed over like the other entries"
    )]
    pub(crate) fn returns(mut self, expected: T) {
        let mode = self.mode.take().expect("a timeline is run only once");
        let world = Rc::new(World::new(&self.given, &self.entries));
        let executor = FakeExecutor(Rc::clone(&world));
        let keyboard = FakeKeyboard(Rc::clone(&world));
        let view = FakeView(Rc::clone(&world));
        let clock = FakeClock(Rc::clone(&world));
        let window_manager = FakeWindowManager;
        let keys_back = [Key::new(Symbol::BACKSPACE)];
        let keys_exit = [Key::new(Symbol::ESCAPE)];
        let keys_retry = [Key::new(Symbol::RETURN)];
        let root_layer = Layer::new("root".to_string());
        let app_specific_layers: Vec<ApplicationConfig> = vec![];
        let context = Context {
            executor: &executor,
            keyboard: &keyboard,
            view: &view,
            clock: &clock,
            window_manager: &window_manager,
            keys_back: &keys_back,
            keys_exit: &keys_exit,
            keys_retry: &keys_retry,
            root_layer: &root_layer,
            app_specific_layers: &app_specific_layers,
        };

        let result = mode(&context);

        world.verify(&self.entries);
        assert_eq!(result, expected, "the mode returned an unexpected result");
    }
}

impl<F> Drop for Timeline<F> {
    fn drop(&mut self) {
        // A panic here while a failed assertion unwinds would abort the whole test binary.
        if !std::thread::panicking() {
            assert!(self.mode.is_none(), "the timeline was never run");
        }
    }
}

/// An entry of the trace, with the virtual moment it happened at.
#[derive(Debug)]
struct Happened {
    at: Duration,
    entry: Entry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandState {
    Idle,
    Running,
    Finished,
    Killed,
    /// A kill failed, so the command is no longer tracked.
    Unknown,
}

/// The state the fakes share and the mode acts on.
#[derive(Debug)]
struct World {
    /// The real moment virtual time is counted from.
    epoch: Instant,
    /// Virtual time since the epoch.
    now: Cell<Duration>,
    /// The events still to happen, in order.
    future: RefCell<VecDeque<(Duration, Event)>>,
    /// Keys pressed and not yet read.
    pending_keys: RefCell<VecDeque<Key>>,
    /// The result the running command finished with, until it is polled.
    pending_finish: RefCell<Option<Result<String, ExecutorError>>>,
    starts_fail: Cell<bool>,
    kills_fail: Cell<bool>,
    command_state: Cell<CommandState>,
    polled_since_read: Cell<bool>,
    trace: RefCell<Vec<Happened>>,
}

impl World {
    fn new(given: &[Event], entries: &[Entry]) -> Self {
        let future = entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::At(at, event) => Some((*at, event.clone())),
                Entry::Effect(_) => None,
            })
            .collect();
        let world = Self {
            epoch: Instant::now(),
            now: Cell::new(Duration::ZERO),
            future: RefCell::new(future),
            pending_keys: RefCell::new(VecDeque::new()),
            pending_finish: RefCell::new(None),
            starts_fail: Cell::new(false),
            kills_fail: Cell::new(false),
            command_state: Cell::new(CommandState::Idle),
            polled_since_read: Cell::new(false),
            trace: RefCell::new(vec![]),
        };
        for event in given {
            world.apply(event.clone());
        }
        world
    }

    fn record(&self, entry: Entry) {
        self.trace.borrow_mut().push(Happened {
            at: self.now.get(),
            entry,
        });
    }

    fn effect(&self, effect: Effect) {
        self.record(Entry::Effect(effect));
    }

    /// Makes every event due by now happen.
    fn catch_up(&self) {
        while self.next_event_by(self.now.get()) {}
    }

    /// Makes the next event happen if it is due by the moment.
    fn next_event_by(&self, moment: Duration) -> bool {
        let due = {
            let mut future = self.future.borrow_mut();
            match future.front() {
                Some((at, _)) if *at <= moment => future.pop_front(),
                _ => None,
            }
        };
        let Some((at, event)) = due else {
            return false;
        };
        self.now.set(at);
        self.record(Entry::At(at, event.clone()));
        self.apply(event);
        true
    }

    fn apply(&self, event: Event) {
        match event {
            Event::KeyPressed(key) => self.pending_keys.borrow_mut().push_back(key),
            Event::CommandFinishes(result) => {
                assert_eq!(
                    self.command_state.get(),
                    CommandState::Running,
                    "timeline: a command finishes at {} ms while none is running",
                    self.now.get().as_millis()
                );
                let previous = self.pending_finish.replace(Some(result));
                assert!(
                    previous.is_none(),
                    "timeline: the command finishes a second time at {} ms",
                    self.now.get().as_millis()
                );
            }
            Event::StartsFail => self.starts_fail.set(true),
            Event::StartsSucceed => self.starts_fail.set(false),
            Event::KillsFail => self.kills_fail.set(true),
        }
    }

    /// Checks that the trace matches the timeline and that nothing was left behind.
    fn verify(&self, entries: &[Entry]) {
        let trace = self.trace.borrow();
        let same = trace.len() == entries.len()
            && trace
                .iter()
                .zip(entries)
                .all(|(happened, entry)| happened.entry == *entry);
        assert!(same, "{}", timeline_mismatch(entries, &trace));
        assert!(
            self.pending_keys.borrow().is_empty(),
            "a key was pressed but never read"
        );
    }
}

fn timeline_mismatch(entries: &[Entry], trace: &[Happened]) -> String {
    let first_difference = entries
        .iter()
        .zip(trace)
        .position(|(entry, happened)| *entry != happened.entry)
        .unwrap_or(entries.len().min(trace.len()));
    let marker = |index: usize| {
        if index == first_difference {
            "  <-- differs"
        } else {
            ""
        }
    };

    let expected: Vec<String> = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| format!("    {}{}", entry.describe(), marker(index)))
        .collect();
    let actual: Vec<String> = trace
        .iter()
        .enumerate()
        .map(|(index, happened)| {
            format!(
                "    {:>6} ms  {}{}",
                happened.at.as_millis(),
                happened.entry.content(),
                marker(index)
            )
        })
        .collect();
    format!(
        "the run did not follow the timeline\n\nexpected:\n{}\n\nactual:\n{}\n",
        expected.join("\n"),
        actual.join("\n")
    )
}

struct FakeExecutor(Rc<World>);

impl Executor for FakeExecutor {
    #[expect(
        clippy::panic_in_result_fn,
        reason = "a protocol violation is a test failure, and only a panic can report one from inside a port call"
    )]
    fn start(&self, command: &str) -> Result<Box<dyn RunningCommand>, ExecutorError> {
        let world = &self.0;
        world.catch_up();
        assert_ne!(
            world.command_state.get(),
            CommandState::Running,
            "protocol: a command was started while another one runs"
        );
        if world.starts_fail.get() {
            return Err(ExecutorError::Startup);
        }
        world.command_state.set(CommandState::Running);
        world.polled_since_read.set(false);
        world.effect(Effect::Started(command.to_string()));
        Ok(Box::new(FakeRunning(Rc::clone(world))))
    }

    fn run_in_background(&self, command: &str) -> Result<(), ExecutorError> {
        self.0.catch_up();
        self.0.effect(Effect::Background(command.to_string()));
        Ok(())
    }
}

#[derive(Debug)]
struct FakeRunning(Rc<World>);

impl RunningCommand for FakeRunning {
    fn poll(self: Box<Self>) -> CommandPoll {
        let world = &self.0;
        world.catch_up();
        assert_eq!(
            world.command_state.get(),
            CommandState::Running,
            "protocol: a command that is not running was polled"
        );
        world.polled_since_read.set(true);
        match world.pending_finish.take() {
            Some(result) => {
                world.command_state.set(CommandState::Finished);
                CommandPoll::Finished(result)
            }
            None => CommandPoll::Running(self),
        }
    }

    fn kill(self: Box<Self>) -> Result<(), ExecutorError> {
        let world = &self.0;
        world.catch_up();
        // A finish nobody polled is gone with the command.
        world.pending_finish.take();
        if world.kills_fail.get() {
            world.command_state.set(CommandState::Unknown);
            return Err(ExecutorError::Unknown);
        }
        world.command_state.set(CommandState::Killed);
        world.effect(Effect::Killed);
        Ok(())
    }
}

impl Drop for FakeRunning {
    fn drop(&mut self) {
        // A panic here while a failed assertion unwinds would abort the whole test binary.
        if !std::thread::panicking() {
            assert_ne!(
                self.0.command_state.get(),
                CommandState::Running,
                "protocol: a running command was dropped without being killed"
            );
        }
    }
}

struct FakeKeyboard(Rc<World>);

impl Keyboard for FakeKeyboard {
    fn capture_any(&self) -> Key {
        let world = &self.0;
        world.catch_up();
        assert_ne!(
            world.command_state.get(),
            CommandState::Running,
            "protocol: the keyboard was read blockingly while a command runs"
        );
        loop {
            if let Some(key) = world.pending_keys.borrow_mut().pop_front() {
                return key;
            }
            assert!(
                world.next_event_by(Duration::MAX),
                "the timeline ran out during a blocking read"
            );
        }
    }

    fn capture_within(&self, timeout: Duration) -> Option<Key> {
        let world = &self.0;
        world.catch_up();
        assert!(
            world.polled_since_read.replace(false),
            "protocol: the keyboard was read twice without a poll in between"
        );
        assert_eq!(
            world.command_state.get(),
            CommandState::Running,
            "protocol: the keyboard was read with a timeout while no command runs"
        );
        let deadline = world.now.get().saturating_add(timeout);
        loop {
            if let Some(key) = world.pending_keys.borrow_mut().pop_front() {
                return Some(key);
            }
            if !world.next_event_by(deadline) {
                world.now.set(deadline);
                return None;
            }
        }
    }
}

struct FakeView(Rc<World>);

impl View for FakeView {
    fn render(&self, model: ViewModel) {
        self.0.catch_up();
        self.0.effect(Effect::Rendered(Screen::from(&model)));
    }
}

struct FakeClock(Rc<World>);

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.0
            .epoch
            .checked_add(self.0.now.get())
            .expect("virtual time overflows the clock")
    }
}

struct FakeWindowManager;

impl WindowManager for FakeWindowManager {
    fn get_active_window_name(&self) -> Option<String> {
        None
    }
}
