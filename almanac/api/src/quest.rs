//! `quest`, `edit` and `remove`: a quest written into the registry, or changed there.

use weft_sdk::{Event, Record, Registry, WithComponents, World, kind};

use crate::events::{CLAIMED, DONE, Done};
use crate::names::{qualified, quest_record, quest_text};
use crate::record::{AFTER, ICON, IN_CHAPTER, Kind, ORDER, QUEST, REWARDS, Reward, TASKS, Task};

/// A quest `name` of the calling mod in chapter `chapter` (`cogwork:cogwork`, any mod's).
/// `Chapter::quest` is the same for a chapter of one's own, in the order they are written.
///
/// ```ignore
/// almanac_api::quest(r, "cogwork:cogwork", "spare_parts")
///     .icon("base:chest").after("cogwork:first_teeth")
///     .have("cogwork:iron_cog", 8).gives("base:diamond", 1);
/// ```
pub fn quest<'a>(registry: &'a mut Registry, chapter: &str, name: &str) -> QuestBuilder<'a> {
    let name = qualified(registry.namespace(), name);
    let values = Values {
        chapter: Some(chapter.to_string()),
        ..Values::default()
    };
    QuestBuilder {
        registry,
        name,
        edit: false,
        values,
        hooks: Vec::new(),
    }
}

/// Changes quest `quest` (`cogwork:wound_up`) of another mod: what is said here is set or added,
/// the rest stays. Load after that mod (`declare_mod!(.., after: ["cogwork"], ..)`); editing a
/// quest nobody registered stops the game from loading, as for anything in the registry, so ask
/// `registry.has_mod(..)` first when that mod may be missing.
///
/// ```ignore
/// almanac_api::edit(r, "cogwork:wound_up").gives("base:diamond", 1).without_reward("cogwork:torque_shaft");
/// ```
pub fn edit<'a>(registry: &'a mut Registry, quest: &str) -> QuestBuilder<'a> {
    let name = qualified(registry.namespace(), quest);
    QuestBuilder {
        registry,
        name,
        edit: true,
        values: Values::default(),
        hooks: Vec::new(),
    }
}

/// Takes quest `quest` (`cogwork:wound_up`) of another mod out of the book.
pub fn remove(registry: &mut Registry, quest: &str) {
    let name = qualified(registry.namespace(), quest);
    registry.remove(Record::generated(quest_record(&name)));
}

/// Written into the registry when dropped, like the SDK's builders.
pub struct QuestBuilder<'a> {
    registry: &'a mut Registry,
    /// `cogwork:wound_up`.
    name: String,
    edit: bool,
    values: Values,
    hooks: Vec<Hook>,
}

/// What a builder was told, to go onto the quest's record.
#[derive(Default)]
struct Values {
    chapter: Option<String>,
    icon: Option<String>,
    order: Option<i32>,
    after: Vec<String>,
    tasks: Vec<Task>,
    rewards: Vec<Reward>,
    without_tasks: Vec<String>,
    without_rewards: Vec<String>,
}

/// What a mod has run on the server for a player: the world they are in, and their id.
type Run = Box<dyn Fn(&World, u64) + Send + Sync>;

/// Code of the calling mod to run for a player when the quest's event comes.
struct Hook {
    event: &'static Event<Done>,
    run: Run,
}

impl QuestBuilder<'_> {
    /// The item its tile shows.
    pub fn icon(mut self, item: &str) -> Self {
        self.values.icon = Some(self.full(item));
        self
    }

    /// Opens only once `quest` is finished (call again for more). A name without a mod is the
    /// calling mod's.
    pub fn after(mut self, quest: &str) -> Self {
        let quest = self.full(quest);
        self.values.after.push(quest);
        self
    }

    /// Where it stands in its chapter: lower comes first.
    pub fn order(mut self, order: i32) -> Self {
        self.values.order = Some(order);
        self
    }

    /// To do: have `count` of `items` (a name, or `#ingots/iron` for any mod's) at once.
    pub fn have(self, items: &str, count: u32) -> Self {
        self.task(Kind::Have, items, count)
    }

    /// To do: place `count` of `blocks`.
    pub fn place(self, blocks: &str, count: u32) -> Self {
        self.task(Kind::Place, blocks, count)
    }

    /// To do: break `count` of `blocks`.
    pub fn dig(self, blocks: &str, count: u32) -> Self {
        self.task(Kind::Dig, blocks, count)
    }

    /// To do: kill `count` of `mobs` (`base:zombie`, `#hostile`).
    pub fn kill(self, mobs: &str, count: u32) -> Self {
        self.task(Kind::Kill, mobs, count)
    }

    /// To do: use `count` of `items`, as food is eaten or a tool put to use.
    pub fn use_item(self, items: &str, count: u32) -> Self {
        self.task(Kind::Use, items, count)
    }

    /// To do: something the mod counts itself with `advance(.., name, ..)`, up to `count`. It is
    /// shown as the translation `quest.<mod>.<quest>.<name>`.
    pub fn count(mut self, name: &str, count: u32) -> Self {
        let text = format!("{}.{name}", quest_text(&self.name));
        self.values.tasks.push(Task::counted(name, count, &text));
        self
    }

    /// The translation shown for the task added last, in place of the one made from its kind:
    /// for a task counting a tag, which has no name to show.
    pub fn told(mut self, text: &str) -> Self {
        if let Some(task) = self.values.tasks.last_mut() {
            task.text = text.to_string();
        }
        self
    }

    /// Hands over `count` of `item` when its rewards are claimed (call again for more).
    pub fn gives(mut self, item: &str, count: u32) -> Self {
        let reward = Reward::new(&self.full(item), count);
        self.values.rewards.push(reward);
        self
    }

    /// Takes task `name` out, in an `edit`: `place/cogwork:spring_engine` in full, or a counted
    /// task's own name. A task the quest doesn't have stops the game from loading.
    pub fn without_task(mut self, name: &str) -> Self {
        self.values.without_tasks.push(name.to_string());
        self
    }

    /// Takes the reward of item `item` out, in an `edit`. A reward the quest doesn't have stops
    /// the game from loading.
    pub fn without_reward(mut self, item: &str) -> Self {
        let item = self.full(item);
        self.values.without_rewards.push(item);
        self
    }

    /// Runs on the server for a player who finishes it, before `on_claim`. The code stays with the calling mod: an
    /// `edit` adds to what the quest's own mod runs, and can't take that away.
    pub fn on_done(mut self, run: impl Fn(&World, u64) + Send + Sync + 'static) -> Self {
        self.hooks.push(Hook {
            event: &DONE,
            run: Box::new(run),
        });
        self
    }

    /// Runs on the server for a player who takes its rewards: a reward that is no item.
    pub fn on_claim(mut self, run: impl Fn(&World, u64) + Send + Sync + 'static) -> Self {
        self.hooks.push(Hook {
            event: &CLAIMED,
            run: Box::new(run),
        });
        self
    }

    pub(crate) fn in_order(mut self, order: i32) -> Self {
        self.values.order.get_or_insert(order);
        self
    }

    fn task(mut self, kind: Kind, what: &str, count: u32) -> Self {
        let task = Task::new(kind, &self.full(what), count);
        self.values.tasks.push(task);
        self
    }

    /// `name` as the calling mod's unless it names a mod.
    fn full(&self, name: &str) -> String {
        qualified(self.registry.namespace(), name)
    }
}

impl Drop for QuestBuilder<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let record = Record::generated(quest_record(&self.name));
        let values = std::mem::take(&mut self.values);
        if self.edit {
            drop(values.onto(self.registry.edit(record)));
        } else {
            drop(values.onto(self.registry.record(record).tag(QUEST.name())));
        }
        for hook in self.hooks.drain(..) {
            hook.listen(self.registry, record);
        }
    }
}

impl Values {
    /// Everything said, as operations on the quest's record.
    fn onto<B: WithComponents<Kind = kind::Record>>(self, mut record: B) -> B {
        if let Some(chapter) = self.chapter {
            record = record.set(&IN_CHAPTER, chapter);
        }
        if let Some(icon) = self.icon {
            record = record.set(&ICON, icon);
        }
        if let Some(order) = self.order {
            record = record.set(&ORDER, order);
        }
        for quest in self.after {
            record = record.push(&AFTER, quest);
        }
        for name in self.without_tasks {
            record = record.remove_item(&TASKS, &name);
        }
        for name in self.without_rewards {
            record = record.remove_item(&REWARDS, &name);
        }
        for task in self.tasks {
            record = record.push(&TASKS, task);
        }
        for reward in self.rewards {
            record = record.push(&REWARDS, reward);
        }
        record
    }
}

impl Hook {
    /// Runs it for every player the event names with the quest of `record`.
    fn listen(self, registry: &mut Registry, record: &'static Record) {
        let Hook { event, run } = self;
        registry.on_event(event, move |batch| {
            let world = batch.world();
            for done in batch.iter().filter(|done| done.quest == record.id()) {
                run(&world, done.player);
            }
        });
    }
}
