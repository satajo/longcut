//! The harness the scenarios run on: fakes of every port of the crate, driven through a scenario.
//!
//! A scenario sets up the world a launch is served in and names the session mode the launcher
//! reports. What follows is a timeline: the events the world throws at the service, among them the
//! virtual time that passes, and between them the effects the service is expected to have on the
//! world. The chain is a declaration; its last call serves the launch on a `CoreService` made of
//! the fakes. The fakes play the events back as the service calls into the ports and record what
//! actually happened in one trace, which must match the timeline entry for entry once the session
//! has ended; an effect the timeline does not list at its place fails the run as it happens. The
//! virtual clock of the run moves only while the keyboard waits for a key or the service sleeps: a
//! timed wait moves it by exactly the timeout the service asked for, a sleep by exactly the
//! duration, and a blocking wait moves it to the moment of each event it waits through, so a
//! scenario runs in no wall time at all.
//!
//! An event happens at its moment, or later once every entry listed before it has happened, so the
//! timeline plays back in order. Time passing is an entry of its own, traced as the clock reaches
//! its moment, and it is never late: an effect listed after it happens once the clock has reached
//! that moment, and an effect listed before it happens before the clock reaches it. A run whose
//! clock reaches the moment before every entry listed ahead of it has happened fails when the
//! time passing comes up to be traced. Only a waiting keyboard or a sleeping service traces time
//! passing, so time passing listed before the last effects of the launch states that the service
//! still serves it at that moment, and a timeline cannot end on time passing. A wait or a sleep
//! reaching far past the last moment of the timeline fails the run as having outlived it.
//!
//! The fakes check the port protocol on every call and panic at the call that breaks it: a timed
//! keyboard read without a poll since the previous read, a blocking read while a command runs, a
//! start while a command runs, a command dropped while still running, the keyboard taken while it
//! is already taken, or the launcher asked for a second launch. A run that ends with the keyboard
//! still taken, or without the launcher asked for the launch, breaks the protocol too.

use longcut_core::config::{ApplicationConfig, Config};
use longcut_core::model::key::{Key, Symbol};
use longcut_core::model::layer::Layer;
use longcut_core::port::clock::Clock;
use longcut_core::port::executor::{CommandPoll, Executor, ExecutorError, RunningCommand};
use longcut_core::port::input::{Input, InputError, Keyboard};
use longcut_core::port::view::{ParameterVariant, View, ViewAction, ViewModel};
use longcut_core::port::{Launcher, WindowManager};
use longcut_core::{CoreService, SessionMode};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// The back key every scenario is configured with.
pub(crate) fn back_key() -> Key {
    Key::new(Symbol::BACKSPACE)
}

/// The exit key every scenario is configured with.
pub(crate) fn exit_key() -> Key {
    Key::new(Symbol::ESCAPE)
}

/// Something the world does to the service.
///
/// An event happens at a moment: the moment the latest time passing on the timeline before it
/// reaches, and the start of the run when no time has passed. A fault holds from its moment on;
/// a fault that can be lifted is lifted by an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    /// Virtual time passes for the number of milliseconds, so the events after it happen that
    /// much later. It happens at the moment the clock reaches once the time has passed, as the
    /// keyboard waits or the service sleeps through it, and never later: the effects listed
    /// before it happen before the clock reaches that moment, those listed after it once the
    /// clock has reached it.
    TimePasses(u64),
    /// The user presses the key. It waits in the keyboard until the service reads it.
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
    /// The keyboard cannot be taken from now on. Nothing lifts this fault.
    KeyboardUnavailable,
}

impl Event {
    /// The event as a report states it.
    fn describe(&self) -> String {
        match self {
            Event::TimePasses(millis) => format!("{millis} ms pass"),
            event @ (Event::KeyPressed(_)
            | Event::CommandFinishes(_)
            | Event::StartsFail
            | Event::StartsSucceed
            | Event::KillsFail
            | Event::ActiveWindow(_)
            | Event::KeyboardUnavailable) => format!("{event:?}"),
        }
    }
}

/// Something the service did to the world through a port.
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

/// What a rendered view model showed, reduced to what the scenarios assert on.
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

/// The world before the launch.
///
/// The back key is `BackSpace` and the exit key Escape. The root layer is an empty one named
/// root, and there are no application-specific layers.
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
        let pattern = regex::Regex::new(pattern).expect("the app layer pattern is a regex");
        self.app_specific_layers.push(ApplicationConfig {
            pattern,
            root_layer: layer,
        });
        self
    }

    /// The event has happened by the time the service starts. Time cannot pass before the start.
    pub(crate) fn given(mut self, event: Event) -> Self {
        assert!(
            !matches!(event, Event::TimePasses(_)),
            "time cannot pass before the launch"
        );
        self.given.push(event);
        self
    }

    /// The service serves one launch from here on, and the launcher reports it in the session
    /// mode. What follows is the timeline of the launch, which begins as the service waits for it.
    pub(crate) fn launch(self, mode: SessionMode) -> Timeline {
        Timeline {
            entries: vec![],
            moment: Duration::ZERO,
            run: Some((self, mode)),
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
            Entry::At(at, event) => format!("at {} ms: {}", at.as_millis(), event.describe()),
            Entry::Effect(effect) => format!("{effect:?}"),
        }
    }

    /// The entry without its moment.
    fn content(&self) -> String {
        match self {
            Entry::At(_, event) => event.describe(),
            Entry::Effect(effect) => format!("{effect:?}"),
        }
    }
}

/// What happens while the launch is served: scripted events at moments counted from the start of
/// the run, time passing among them, and the effects expected between them. The timeline is told
/// by its last call, `ends`.
#[must_use]
pub(crate) struct Timeline {
    entries: Vec<Entry>,
    /// The moment the timeline has reached: the moment of its latest time passing.
    moment: Duration,
    /// The world before the run and the session mode launched in it, until it has been run.
    run: Option<(Scenario, SessionMode)>,
}

impl Timeline {
    /// The event happens at its moment, or as soon after it as every entry listed before it has
    /// happened. Its moment is the one the latest time passing before it reaches. Time passing
    /// moves the moment of the timeline on, and it is never late: it happens as the clock reaches
    /// its moment, so an effect expected after it happens after the clock has reached that moment
    /// and before the next entry, and one expected before it happens before the clock reaches it.
    /// Blocking reads do not care when a key lands, so a key press that only has to follow the
    /// previous entries is written without time passing before it.
    pub(crate) fn then(mut self, event: Event) -> Self {
        if let Event::TimePasses(millis) = event {
            assert_ne!(millis, 0, "time passing lasts at least 1 ms");
            self.moment = self
                .moment
                .checked_add(Duration::from_millis(millis))
                .expect("the timeline overflows virtual time");
        }
        self.entries.push(Entry::At(self.moment, event));
        self
    }

    /// The service is expected to have had the effect before the next entry happens.
    pub(crate) fn expect(mut self, effect: Effect) -> Self {
        self.entries.push(Entry::Effect(effect));
        self
    }

    /// The session ends at the end of the timeline.
    ///
    /// Serves the launch on a `CoreService` made of the fakes and configured as the scenario set
    /// up, then checks that the service asked the launcher for the launch and that the run
    /// followed the timeline.
    #[track_caller]
    pub(crate) fn ends(mut self) {
        let (scenario, mode) = self.run.take().expect("a timeline is run only once");
        assert!(
            !matches!(
                self.entries.last(),
                Some(Entry::At(_, Event::TimePasses(_)))
            ),
            "the timeline ends on time passing, which only a running session can trace"
        );
        let world = Rc::new(World::new(
            &scenario.given,
            std::mem::take(&mut self.entries),
            self.moment,
        ));
        let config = Config {
            keys_back: vec![back_key()],
            keys_exit: vec![exit_key()],
            root_layer: scenario.root_layer,
            app_specific_layers: scenario.app_specific_layers,
        };
        let executor = FakeExecutor(Rc::clone(&world));
        let input = FakeInput(Rc::clone(&world));
        let view = FakeView(Rc::clone(&world));
        let clock = FakeClock(Rc::clone(&world));
        let window_manager = FakeWindowManager(Rc::clone(&world));
        let launcher = FakeLauncher {
            world: Rc::clone(&world),
            launch: Cell::new(Some(mode)),
        };
        let service = CoreService::new(&executor, &input, &view, &clock, &window_manager, config);

        service.serve_launch(&launcher);

        assert!(
            launcher.launch.get().is_none(),
            "the service never waited for the launch"
        );
        world.verify();
    }
}

impl Drop for Timeline {
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

/// The state the fakes share and the service acts on.
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
    /// The events still to happen, in order, time passing among them.
    future: RefCell<VecDeque<(Duration, Event)>>,
    /// Keys pressed and not yet read.
    pending_keys: RefCell<VecDeque<Key>>,
    /// The result the running command finished with, until it is polled.
    pending_finish: RefCell<Option<Result<String, ExecutorError>>>,
    starts_fail: Cell<bool>,
    kills_fail: Cell<bool>,
    active_window: RefCell<Option<String>>,
    keyboard_unavailable: Cell<bool>,
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
            keyboard_unavailable: Cell::new(false),
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

    /// Records the effect and fails the run when the timeline lists something else at its place,
    /// since the trace can then never match the timeline again.
    fn effect(&self, effect: Effect) {
        let entry = Entry::Effect(effect);
        let listed_here = self.entries.get(self.trace.borrow().len()) == Some(&entry);
        self.record(entry);
        assert!(
            listed_here,
            "the service had an effect the timeline does not list at its place\n\n{}",
            self.mismatch()
        );
    }

    /// Makes every event due by now happen, up to the first one still blocked.
    fn catch_up(&self) {
        while self.next_event_by(self.now.get()) == NextEvent::Happened {}
    }

    /// Makes the next event happen if it is due by the moment and every entry listed before it
    /// has happened. An event happening after its moment happens at the present moment; time
    /// passing moves the clock to its moment and fails the run when the clock is at or past it
    /// already.
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
            assert!(
                !matches!(event, Event::TimePasses(_)) || self.now.get() < *at,
                "the clock reached {} ms before every entry listed ahead of it had happened\n\n{}",
                at.as_millis(),
                self.mismatch()
            );
            let next = (*at, event.clone());
            future.pop_front();
            next
        };
        self.now.set(self.now.get().max(at));
        self.record(Entry::At(at, event.clone()));
        self.apply(event);
        NextEvent::Happened
    }

    /// Takes one step of a wait that lasts until the deadline: makes the next event happen if it
    /// is due by the deadline and listed next, and returns whether it did. When it does not, the
    /// wait has reached its deadline, and the clock moves to it.
    fn event_happens_by(&self, deadline: Duration) -> bool {
        /// How far past the end of the timeline a wait may reach before the run is taken to have
        /// left the timeline for good.
        const OUTLIVE_LIMIT: Duration = Duration::from_mins(1);

        if self.next_event_by(deadline) == NextEvent::Happened {
            return true;
        }
        assert!(
            deadline <= self.end.saturating_add(OUTLIVE_LIMIT),
            "the run outlived the timeline\n\n{}",
            self.mismatch()
        );
        self.now.set(deadline);
        false
    }

    /// The trace so far set against the timeline.
    fn mismatch(&self) -> String {
        timeline_mismatch(&self.entries, &self.trace.borrow())
    }

    fn apply(&self, event: Event) {
        match event {
            // The clock stands at the moment the time has passed to already.
            Event::TimePasses(_) => {}
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
            Event::KeyboardUnavailable => self.keyboard_unavailable.set(true),
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
            !world.keyboard_taken.get(),
            "protocol: the keyboard was taken while it is already taken"
        );
        if world.keyboard_unavailable.get() {
            return Err(InputError {
                reason: "another program holds the keyboard".to_string(),
            });
        }
        world.keyboard_taken.set(true);
        world.effect(Effect::KeyboardTaken);
        Ok(Box::new(FakeKeyboard(Rc::clone(world))))
    }
}

/// The keyboard while the input has it taken. Dropping it gives the keyboard back.
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
            let next = world.next_event_by(Duration::MAX);
            assert!(
                next != NextEvent::Blocked,
                "the service waits for a key, but the timeline expects an effect first\n\n{}",
                world.mismatch()
            );
            assert!(
                next == NextEvent::Happened,
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
            if !world.event_happens_by(deadline) {
                return None;
            }
        }
    }
}

impl Drop for FakeKeyboard {
    fn drop(&mut self) {
        // A panic here while a failed assertion unwinds would abort the whole test binary.
        if !std::thread::panicking() {
            let world = &self.0;
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

    /// Waits through the duration as a timed keyboard wait does, without a keyboard: a key
    /// pressed meanwhile stays unread.
    fn sleep(&self, duration: Duration) {
        let world = &self.0;
        world.catch_up();
        let deadline = world.now.get().saturating_add(duration);
        while world.event_happens_by(deadline) {}
    }
}

struct FakeWindowManager(Rc<World>);

impl WindowManager for FakeWindowManager {
    fn get_active_window_name(&self) -> Option<String> {
        self.0.catch_up();
        self.0.active_window.borrow().clone()
    }
}
