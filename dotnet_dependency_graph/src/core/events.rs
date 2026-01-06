use dotnet_dependency_parser::graph::{DependencyId, Framework};
use std::collections::HashSet;

/// Events representing user interactions. Widgets should publish these
/// instead of mutating state directly. The controller processes events
/// and updates the authoritative state.
#[derive(Debug, Clone)]
pub(crate) enum InteractionEvent {
    Select(DependencyId),
    Highlight(DependencyId),
    SelectFramework(Framework),
}

/// Holds data about interactions on the graph.
#[derive(Default, Debug, Clone)]
struct InteractionState {
    selected: Option<DependencyId>,
    highlighted: Option<DependencyId>,
    selected_framework: Option<Framework>,
    pan_to_dependency: Option<DependencyId>,
}

/// Controller that collects interaction events and applies them to the
/// internal InteractionState. Widgets should publish events and read
/// the state through this controller.
#[derive(Default, Debug)]
pub(crate) struct InteractionController {
    state: InteractionState,
    pending: Vec<InteractionEvent>,
}

impl InteractionController {
    /// Publish an event which will be processed when `process_pending`
    /// is called (usually between UI panels in a frame).
    pub(crate) fn publish(&mut self, ev: InteractionEvent) {
        self.pending.push(ev);
    }

    /// Apply pending events to the state. Events are applied in order.
    ///
    /// Resets highlighted and pan_to_dependency.
    pub(crate) fn process_pending(&mut self, visible_nodes: &mut HashSet<DependencyId>) {
        self.state.highlighted = None;
        self.state.pan_to_dependency = None;

        for ev in self.pending.drain(..) {
            match ev {
                InteractionEvent::Select(opt) => {
                    self.state.selected = Some(opt);
                    self.state.pan_to_dependency = Some(opt);
                }
                InteractionEvent::Highlight(opt) => self.state.highlighted = Some(opt),
                InteractionEvent::SelectFramework(opt) => self.state.selected_framework = Some(opt),
            }
        }

        if let Some(selected) = self.state.selected.or(self.state.pan_to_dependency) {
            visible_nodes.insert(selected);
        }
    }

    // Read-only accessors
    pub(crate) fn selected_dependency(&self) -> Option<DependencyId> {
        self.state.selected
    }

    pub(crate) fn selected_framework(&self) -> Option<&Framework> {
        self.state.selected_framework.as_ref()
    }

    pub(crate) fn highlighted_dependency(&self) -> Option<DependencyId> {
        self.state.highlighted
    }

    pub(crate) fn panned_dependency(&self) -> Option<DependencyId> {
        self.state.pan_to_dependency
    }
}
