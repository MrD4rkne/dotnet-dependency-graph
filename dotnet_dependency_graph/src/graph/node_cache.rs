use dotnet_dependency_parser::graph::DependencyId;
use eframe::egui::{Pos2, Rect, Vec2};
use std::collections::{HashMap, HashSet};

pub(crate) struct CachedNodeData {
    pub(crate) initial_position: Pos2,
    pub(crate) unzoomed_width: f32,
    pub(crate) unzoomed_height: f32,
    pub(crate) rect: Rect,
}

impl CachedNodeData {
    pub(crate) fn new(screen_pos: Pos2, unzoomed_width: f32, unzoomed_height: f32) -> Self {
        Self {
            initial_position: screen_pos,
            unzoomed_width,
            unzoomed_height,
            rect: Rect::from_center_size(Pos2::ZERO, Vec2::new(unzoomed_width, unzoomed_height)),
        }
    }
}

pub(crate) struct NodeCacheManager {
    cache: Option<HashMap<DependencyId, CachedNodeData>>,
}

impl Default for NodeCacheManager {
    fn default() -> Self {
        NodeCacheManager::new()
    }
}

impl NodeCacheManager {
    fn new() -> Self {
        Self { cache: None }
    }

    pub(crate) fn get_or_compute(
        &mut self,
        graph: &dotnet_dependency_parser::graph::DependencyGraph,
        positions: &HashMap<DependencyId, (f32, f32)>,
        visible_nodes: &HashSet<DependencyId>,
        zoom: f32,
        pan_offset: eframe::egui::Vec2,
    ) -> &mut HashMap<DependencyId, CachedNodeData> {
        if self.cache.is_none() {
            let cache = crate::graph::graph_widget::compute_nodes_cache(
                graph,
                positions,
                visible_nodes,
                zoom,
                pan_offset,
            );
            self.cache = Some(cache);
        }
        self.cache.as_mut().unwrap()
    }

    pub(crate) fn invalidate(&mut self) {
        self.cache = None;
    }
}
