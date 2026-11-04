//! Fakes of the ports for testing the modes and the service, driven through a scenario.
//!
//! A scenario sets up the state the mode starts in and names the mode to run, or the session mode
//! of a launch for the service to serve. What follows is a timeline: the events the world throws
//! at the mode, the virtual time that passes between them, and between them the effects the mode
//! is expected to have on the world. The chain is a declaration; its last call runs the mode. The
//! fakes play the events back as the mode calls into the ports and record what actually happened
//! in one trace, which must match the timeline entry for entry once the mode returns. The virtual
//! clock of the run moves only while the keyboard waits for a key, by exactly the timeout the mode
//! asked for, so a test runs in no wall time at all. An event happens at its moment, or later once
//! every entry listed before it has happened, so the timeline plays back in order.
//!
//! The fakes check the port protocol on every call and panic at the call that breaks it: a timed
//! keyboard read without a poll since the previous read, a blocking read while a command runs, a
//! start while a command runs, a command dropped while still running, the keyboard taken while it
//! is already taken, or the launcher asked for a second launch. A run that ends with the keyboard
//! still taken breaks the protocol too.

use crate::config::{ApplicationConfig, Config};
use crate::logic::Context;
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::model::session::SessionMode;
use crate::port::clock::Clock;
use crate::port::executor::{CommandPoll, Executor, ExecutorError, RunningCommand};
use crate::port::input::{Input, InputError, Keyboard};
use crate::port::view::{ParameterVariant, View, ViewAction, ViewModel};
use crate::port::{Launcher, WindowManager};
use crate::service::CoreService;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Something the world does to the mode.
///
/// An event happens at a moment: the moment of the event before it on the timeline, later by the
/// time that passes between them, and the first event at the start of the run. A fault holds from
/// its moment on until an event lifts it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    /// Virtual time passes, so the events after it happen that much later. The mode cannot
    /// observe it: it moves the moments of the events after it and is no entry of the trace.
    TimePasses(Duration),
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
    /// The window manager reports the window as the active one from now on. Until the first such
    /// event it reports none.
    ActiveWindow(String),
}

/// Something the mode did to the world through a port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    Started(String),
    Background(String),
    Rendered(Screen),
    Killed,
    /// The keyboard is taken, so the presses from here on reach longcut only.
    KeyboardTaken,
    /// The taken keyboard is given back.
    KeyboardReleased,
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
    LayerNavigation {
        /// The names of the layers navigated into, root first.
        layer_stack: Vec<String>,
    },
    ParameterInput {
        parameter_name: String,
        input: ParameterScreen,
    },
    None,
}

/// What a parameter input screen asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParameterScreen {
    Character,
    /// The text typed so far.
    Text(String),
    /// The options as (key, option) pairs in the order shown, each key as it displays.
    Choose(Vec<(String, String)>),
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
            ViewModel::LayerNavigation(model) => Screen::LayerNavigation {
                layer_stack: model
                    .layer_stack
                    .iter()
                    .map(|layer| layer.name.clone())
                    .collect(),
            },
            ViewModel::ParameterInput(model) => Screen::ParameterInput {
                parameter_name: model.parameter_name.to_string(),
                input: match &model.parameter {
                    ParameterVariant::Character => ParameterScreen::Character,
                    ParameterVariant::Text { input_value } => {
                        ParameterScreen::Text((*input_value).to_string())
                    }
                    ParameterVariant::Choose { options } => ParameterScreen::Choose(
                        options
                            .iter()
                            .map(|(key, option)| (key.to_string(), (*option).to_string()))
                            .collect(),
                    ),
                },
            },
        }
    }
}

/// The world before the mode runs.
///
/// `BackSpace` is the back key, Escape the exit key and Return the retry key. The root layer is
/// an empty one named root, and there are no application-specific layers.
#[must_use]
pub(crate) struct Scenario {
    given: Vec<Event>,
    root_layer: Layer,
    app_specific_layers: Vec<ApplicationConfig>,
}

impl Scenario {
    pub(crate) fn new() -> Self {
        Self {
            given: vec![],
            root_layer: Layer::new("root".to_string()),
            app_specific_layers: vec![],
        }
    }

    /// The layer is the root layer.
    pub(crate) fn root_layer(mut self, layer: Layer) -> Self {
        self.root_layer = layer;
        self
    }

    /// The layer is the root layer for an active window whose name matches the pattern, after the
    /// application-specific layers given before it.
    pub(crate) fn app_layer(mut self, pattern: &str, layer: Layer) -> Self {
        let pattern = regex::Regex::new(pattern).unwrap_or_else(|error| {
            panic!("the app layer pattern {pattern:?} is invalid: {error}")
        });
        self.app_specific_layers.push(ApplicationConfig {
            pattern,
            root_layer: layer,
        });
        self
    }

    /// The event has happened by the time the mode starts. Time cannot pass before the start.
    pub(crate) fn given(mut self, event: Event) -> Self {
        assert!(
            !matches!(event, Event::TimePasses(_)),
            "time passing is no given event"
        );
        self.given.push(event);
        self
    }

    /// The mode runs from here on. What follows is its timeline.
    pub(crate) fn run<F, T>(self, mode: F) -> Timeline<Mode<F>>
    where
        F: FnOnce(&Context<'_>) -> T,
    {
        Timeline::new(self, Mode(mode))
    }

    /// The service serves one launch from here on, and the launcher reports it in the session
    /// mode. What follows is the timeline of the launch, which begins as the service waits for it.
    pub(crate) fn launch(self, mode: SessionMode) -> Timeline<Launch> {
        Timeline::new(self, Launch(mode))
    }
}

/// What a timeline runs on the fakes.
pub(crate) trait Subject {
    type Output;

    /// Runs on the fakes, configured as the scenario set up.
    fn run(self, config: Config, fakes: &Fakes) -> Self::Output;
}

/// A mode, run on a context made of the fakes.
pub(crate) struct Mode<F>(F);

impl<F, T> Subject for Mode<F>
where
    F: FnOnce(&Context<'_>) -> T,
{
    type Output = T;

    fn run(self, config: Config, fakes: &Fakes) -> T {
        let keyboard = FakeKeyboard(Rc::clone(&fakes.world));
        let keys_retry = [Key::new(Symbol::RETURN)];
        let context = Context {
            executor: &fakes.executor,
            keyboard: &keyboard,
            view: &fakes.view,
            clock: &fakes.clock,
            window_manager: &fakes.window_manager,
            keys_back: &config.keys_back,
            keys_exit: &config.keys_exit,
            keys_retry: &keys_retry,
            root_layer: &config.root_layer,
            app_specific_layers: &config.app_specific_layers,
        };
        (self.0)(&context)
    }
}

/// The service serving one launch in the session mode, made of the fakes and a launcher that
/// reports the launch.
pub(crate) struct Launch(SessionMode);

impl Subject for Launch {
    type Output = ();

    fn run(self, config: Config, fakes: &Fakes) {
        let input = FakeInput(Rc::clone(&fakes.world));
        let launcher = FakeLauncher {
            world: Rc::clone(&fakes.world),
            launch: Cell::new(Some(self.0)),
        };
        let service = CoreService::new(
            &fakes.executor,
            &input,
            &fakes.view,
            &fakes.clock,
            &fakes.window_manager,
            config,
        );

        service.serve_launch(&launcher);

        assert!(
            launcher.launch.get().is_none(),
            "the service never waited for the launch"
        );
    }
}

/// The fakes of the ports every run uses, sharing one world.
pub(crate) struct Fakes {
    world: Rc<World>,
    executor: FakeExecutor,
    view: FakeView,
    clock: FakeClock,
    window_manager: FakeWindowManager,
}

impl Fakes {
    fn new(world: &Rc<World>) -> Self {
        Self {
            world: Rc::clone(world),
            executor: FakeExecutor(Rc::clone(world)),
            view: FakeView(Rc::clone(world)),
            clock: FakeClock(Rc::clone(world)),
            window_manager: FakeWindowManager(Rc::clone(world)),
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
pub(crate) struct Timeline<S> {
    entries: Vec<Entry>,
    /// The moment the timeline has reached: the moment of its latest event, later by the time
    /// that has passed since.
    moment: Duration,
    /// The world before the run and what runs in it, until it has been run.
    run: Option<(Scenario, S)>,
}

impl<S> Timeline<S> {
    fn new(scenario: Scenario, subject: S) -> Self {
        Self {
            entries: vec![],
            moment: Duration::ZERO,
            run: Some((scenario, subject)),
        }
    }

    /// The event happens at its moment, or as soon after it as every entry listed before it has
    /// happened. Its moment is that of the previous event on the timeline, later by the time that
    /// passes in between. Blocking modes do not care when a key lands, so a key press that only
    /// has to follow the previous entries is written without time passing before it.
    pub(crate) fn then(mut self, event: Event) -> Self {
        match event {
            Event::TimePasses(duration) => {
                self.moment = self
                    .moment
                    .checked_add(duration)
                    .expect("the timeline overflows virtual time");
            }
            Event::KeyPressed(_)
            | Event::CommandFinishes(_)
            | Event::StartsFail
            | Event::StartsSucceed
            | Event::KillsFail
            | Event::ActiveWindow(_) => self.entries.push(Entry::At(self.moment, event)),
        }
        self
    }

    /// The mode is expected to have had the effect before the next event happens.
    pub(crate) fn expect(mut self, effect: Effect) -> Self {
        self.entries.push(Entry::Effect(effect));
        self
    }
}

impl<S> Timeline<S>
where
    S: Subject,
    S::Output: PartialEq + Debug,
{
    /// The mode returns the expected result at the end of the timeline.
    ///
    /// Runs the mode, checks that the run followed the timeline and that the mode returned the
    /// expected result.
    #[track_caller]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "the expected result is the last entry of the timeline and is handed over like the other entries"
    )]
    pub(crate) fn returns(mut self, expected: S::Output) {
        let (scenario, subject) = self.run.take().expect("a timeline is run only once");
        let world = Rc::new(World::new(
            &scenario.given,
            std::mem::take(&mut self.entries),
            self.moment,
        ));
        let config = Config {
            keys_back: vec![Key::new(Symbol::BACKSPACE)],
            keys_exit: vec![Key::new(Symbol::ESCAPE)],
            root_layer: scenario.root_layer,
            app_specific_layers: scenario.app_specific_layers,
        };

        let result = subject.run(config, &Fakes::new(&world));

        world.verify();
        assert_eq!(result, expected, "the mode returned an unexpected result");
    }
}

impl<S> Drop for Timeline<S> {
    fn drop(&mut self) {
        // A panic here while a failed assertion unwinds would abort the whole test binary.
        if !std::thread::panicking() {
            assert!(self.run.is_none(), "the timeline was never run");
        }
    }
}

/// What became of the next event when it was given the chance to happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextEvent {
    Happened,
    /// Its moment has not come.
    NotDue,
    /// Its moment has come, but an entry listed before it has not happened.
    Blocked,
    /// The timeline has no events left.
    NoneLeft,
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
    /// The timeline the run must follow.
    entries: Vec<Entry>,
    /// The moment the timeline reaches at its end.
    end: Duration,
    /// The events still to happen, in order.
    future: RefCell<VecDeque<(Duration, Event)>>,
    /// Keys pressed and not yet read.
    pending_keys: RefCell<VecDeque<Key>>,
    /// The result the running command finished with, until it is polled.
    pending_finish: RefCell<Option<Result<String, ExecutorError>>>,
    starts_fail: Cell<bool>,
    kills_fail: Cell<bool>,
    active_window: RefCell<Option<String>>,
    command_state: Cell<CommandState>,
    polled_since_read: Cell<bool>,
    /// Whether the keyboard is taken and not yet given back.
    keyboard_taken: Cell<bool>,
    trace: RefCell<Vec<Happened>>,
}

impl World {
    fn new(given: &[Event], entries: Vec<Entry>, end: Duration) -> Self {
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
            entries,
            end,
            future: RefCell::new(future),
            pending_keys: RefCell::new(VecDeque::new()),
            pending_finish: RefCell::new(None),
            starts_fail: Cell::new(false),
            kills_fail: Cell::new(false),
            active_window: RefCell::new(None),
            command_state: Cell::new(CommandState::Idle),
            polled_since_read: Cell::new(false),
            keyboard_taken: Cell::new(false),
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

    /// Makes every event due by now happen, up to the first one still blocked.
    fn catch_up(&self) {
        while self.next_event_by(self.now.get()) == NextEvent::Happened {}
    }

    /// Makes the next event happen if it is due by the moment and every entry listed before it
    /// has happened. An event happening after its moment happens at the present moment.
    fn next_event_by(&self, moment: Duration) -> NextEvent {
        let (at, event) = {
            let mut future = self.future.borrow_mut();
            let Some((at, event)) = future.front() else {
                return NextEvent::NoneLeft;
            };
            if *at > moment {
                return NextEvent::NotDue;
            }
            let listed_next = matches!(
                self.entries.get(self.trace.borrow().len()),
                Some(Entry::At(listed_at, listed_event)) if listed_at == at && listed_event == event
            );
            if !listed_next {
                return NextEvent::Blocked;
            }
            let next = (*at, event.clone());
            future.pop_front();
            next
        };
        self.now.set(self.now.get().max(at));
        self.record(Entry::At(at, event.clone()));
        self.apply(event);
        NextEvent::Happened
    }

    /// The trace so far set against the timeline.
    fn mismatch(&self) -> String {
        timeline_mismatch(&self.entries, &self.trace.borrow())
    }

    fn apply(&self, event: Event) {
        match event {
            Event::TimePasses(_) => panic!("time passing is no event to apply"),
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
            Event::ActiveWindow(name) => {
                self.active_window.replace(Some(name));
            }
        }
    }

    /// Checks that the trace matches the timeline and that nothing was left behind.
    fn verify(&self) {
        assert!(
            !self.keyboard_taken.get(),
            "protocol: the keyboard was taken and never given back"
        );
        let trace = self.trace.borrow();
        let same = trace.len() == self.entries.len()
            && trace
                .iter()
                .zip(&self.entries)
                .all(|(happened, entry)| happened.entry == *entry);
        assert!(same, "{}", timeline_mismatch(&self.entries, &trace));
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
            match world.next_event_by(Duration::MAX) {
                NextEvent::Happened => {}
                NextEvent::Blocked => panic!(
                    "the mode waits for a key, but the timeline expects an effect first\n\n{}",
                    world.mismatch()
                ),
                NextEvent::NotDue | NextEvent::NoneLeft => {
                    panic!("the timeline ran out during a blocking read")
                }
            }
        }
    }

    fn capture_within(&self, timeout: Duration) -> Option<Key> {
        /// How far past the end of the timeline a wait may reach before the run is taken to have
        /// left the timeline for good.
        const OUTLIVE_LIMIT: Duration = Duration::from_mins(1);

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
            if world.next_event_by(deadline) != NextEvent::Happened {
                assert!(
                    deadline <= world.end.saturating_add(OUTLIVE_LIMIT),
                    "the run outlived the timeline\n\n{}",
                    world.mismatch()
                );
                world.now.set(deadline);
                return None;
            }
        }
    }
}

struct FakeInput(Rc<World>);

impl Input for FakeInput {
    #[expect(
        clippy::panic_in_result_fn,
        reason = "a protocol violation is a test failure, and only a panic can report one from inside a port call"
    )]
    fn take_keyboard(&self) -> Result<Box<dyn Keyboard + '_>, InputError> {
        let world = &self.0;
        world.catch_up();
        assert!(
            !world.keyboard_taken.replace(true),
            "protocol: the keyboard was taken while it is already taken"
        );
        world.effect(Effect::KeyboardTaken);
        Ok(Box::new(TakenKeyboard(FakeKeyboard(Rc::clone(world)))))
    }
}

/// The keyboard while the input has it taken. Dropping it gives the keyboard back.
struct TakenKeyboard(FakeKeyboard);

impl Keyboard for TakenKeyboard {
    fn capture_any(&self) -> Key {
        self.0.capture_any()
    }

    fn capture_within(&self, timeout: Duration) -> Option<Key> {
        self.0.capture_within(timeout)
    }
}

impl Drop for TakenKeyboard {
    fn drop(&mut self) {
        // A panic here while a failed assertion unwinds would abort the whole test binary.
        if !std::thread::panicking() {
            let world = &self.0.0;
            world.catch_up();
            world.keyboard_taken.set(false);
            world.effect(Effect::KeyboardReleased);
        }
    }
}

/// Reports one launch, then none.
struct FakeLauncher {
    world: Rc<World>,
    /// The launch to report, until it has been reported.
    launch: Cell<Option<SessionMode>>,
}

impl Launcher for FakeLauncher {
    fn wait_for_launch(&self) -> SessionMode {
        self.world.catch_up();
        self.launch
            .take()
            .expect("protocol: the launcher was asked for a second launch")
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

struct FakeWindowManager(Rc<World>);

impl WindowManager for FakeWindowManager {
    fn get_active_window_name(&self) -> Option<String> {
        self.0.catch_up();
        self.0.active_window.borrow().clone()
    }
}
