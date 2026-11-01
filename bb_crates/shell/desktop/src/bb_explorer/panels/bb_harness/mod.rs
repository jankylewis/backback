mod actor;
#[cfg(test)]
#[path = "view/tests/mod.rs"]
mod bb_tests;
#[cfg(debug_assertions)]
#[path = "view/demo.rs"]
pub(crate) mod demo;
#[path = "view/glue.rs"]
mod glue;
#[path = "bb_model/mod.rs"]
mod model;
mod view;

use gpui::{ScrollStrategy, UniformListScrollHandle};
use std::path::Path;

use actor::{Actor, Command, Event};
use model::{arguments, discover, tool_views, Server, ServerStatus};

pub(crate) use glue::{install_poll, visible_panel, ToggleHarness};
pub(crate) use model::HarnessError;

#[derive(Clone, Copy)]
pub(crate) enum HarnessVisibility {
    Hidden,
    Visible,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tab {
    Tools,
    Log,
}

pub(crate) struct Harness {
    pub(crate) servers: Vec<Server>,
    actors: Vec<Actor>,
    pub(crate) selected: Option<usize>,
    pub(crate) tool: Option<usize>,
    pub(crate) tab: Tab,
    pub(crate) follow: bool,
    pub(crate) search: String,
    pub(crate) edit: Option<usize>,
    pub(crate) search_edit: bool,
    pub(crate) log_scroll: UniformListScrollHandle,
    pub(crate) result_scroll: UniformListScrollHandle,
}

impl Harness {
    pub(crate) fn load(root: &Path) -> Result<Self, model::HarnessError> {
        let servers: Vec<Server> = discover(root)?;
        let actors: Vec<Actor> = servers.iter().map(|_: &Server| Actor::new()).collect();
        Ok(Self {
            servers,
            actors,
            selected: None,
            tool: None,
            tab: Tab::Tools,
            follow: true,
            search: String::new(),
            edit: None,
            search_edit: false,
            log_scroll: UniformListScrollHandle::new(),
            result_scroll: UniformListScrollHandle::new(),
        })
    }

    pub(crate) fn empty() -> Self {
        Self {
            servers: Vec::new(),
            actors: Vec::new(),
            selected: None,
            tool: None,
            tab: Tab::Tools,
            follow: true,
            search: String::new(),
            edit: None,
            search_edit: false,
            log_scroll: UniformListScrollHandle::new(),
            result_scroll: UniformListScrollHandle::new(),
        }
    }

    pub(crate) fn start(&mut self, index: usize) {
        let Some(server): Option<&mut Server> = self.servers.get_mut(index) else {
            return;
        };
        let Some(actor): Option<&Actor> = self.actors.get(index) else {
            return;
        };
        server.generation = server.generation.saturating_add(1);
        server.status = ServerStatus::Starting;
        server.tools.clear();
        server.error = None;
        server.result = None;
        server.pid = None;
        let _sent: bool = actor
            .send
            .send(Command::Start {
                project: server.path.clone(),
                generation: server.generation,
            })
            .is_ok();
    }

    pub(crate) fn stop(&mut self, index: usize) {
        let Some(server): Option<&mut Server> = self.servers.get_mut(index) else {
            return;
        };
        let Some(actor): Option<&Actor> = self.actors.get(index) else {
            return;
        };
        server.generation = server.generation.saturating_add(1);
        server.status = ServerStatus::Stopping;
        let _sent: bool = actor
            .send
            .send(Command::Stop {
                generation: server.generation,
            })
            .is_ok();
    }

    pub(crate) fn call(&mut self) {
        let (Some(index), Some(tool_index)): (Option<usize>, Option<usize>) =
            (self.selected, self.tool)
        else {
            return;
        };
        let Some(server): Option<&mut Server> = self.servers.get_mut(index) else {
            return;
        };
        if server.status != ServerStatus::Running {
            return;
        }
        let Some(tool): Option<&model::ToolView> = server.tools.get(tool_index) else {
            return;
        };
        match arguments(&tool.fields) {
            Ok(arguments) => {
                if let Some(actor) = self.actors.get(index) {
                    let _sent: bool = actor
                        .send
                        .send(Command::Call {
                            name: tool.name.clone(),
                            arguments,
                            generation: server.generation,
                        })
                        .is_ok();
                    server.result = Some(String::from("Calling…").into());
                    server.error = None;
                }
            }
            Err(error) => server.error = Some(error.to_string()),
        }
    }

    pub(crate) fn poll(&mut self) -> bool {
        let mut changed: bool = false;
        for (index, actor) in self.actors.iter().enumerate() {
            while let Ok(event) = actor.receive.try_recv() {
                let Some(server): Option<&mut Server> = self.servers.get_mut(index) else {
                    continue;
                };
                let event_generation: u64 = match &event {
                    Event::Started { generation, .. }
                    | Event::Stopped { generation }
                    | Event::Crashed { generation, .. }
                    | Event::Called { generation, .. }
                    | Event::Log { generation, .. } => *generation,
                };
                if event_generation != server.generation {
                    continue;
                }
                changed = true;
                match event {
                    Event::Started { tools, pid, .. } => {
                        server.tools = tool_views(tools);
                        server.status = ServerStatus::Running;
                        server.pid = pid;
                    }
                    Event::Stopped { .. } => {
                        server.status = ServerStatus::Stopped;
                        server.pid = None;
                    }
                    Event::Crashed { error, .. } => {
                        server.status = ServerStatus::Crashed(error.clone());
                        server.error = Some(error);
                        server.pid = None;
                    }
                    Event::Called { text, is_error, .. } => {
                        server.result = Some(text);
                        server.error = is_error.then(|| "tool returned an error".into());
                    }
                    Event::Log { line, .. } => {
                        server.logs.push(line);
                        if self.follow && self.selected == Some(index) {
                            let last: usize = server.logs.len().saturating_sub(1);
                            self.log_scroll.scroll_to_item(last, ScrollStrategy::Bottom);
                        }
                    }
                }
            }
        }
        changed
    }
}
