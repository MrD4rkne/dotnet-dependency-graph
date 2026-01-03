use crate::session::Session;
use anyhow::Error;
use dotnet_dependency_parser::graph::{DependencyId, SerializableGraph};
use std::path::PathBuf;

pub(crate) fn save_state(session: &Session, path: PathBuf) -> Result<(), Error> {
    // Build metadata map keyed by DependencyId so parser can remap ids when loading
    let metadata: std::collections::HashMap<
        dotnet_dependency_parser::graph::DependencyId,
        (bool, (f32, f32)),
    > = session
        .cache
        .node_cache()
        .iter()
        .map(|(id, cache)| {
            (
                *id,
                (
                    session.visible_nodes.contains(id),
                    (cache.position().x, cache.position().y),
                ),
            )
        })
        .collect();

    let serializable = session
        .graph
        .clone()
        .try_into_serializable(Some(metadata))
        .map_err(|e| anyhow::anyhow!("Failed to create serializable graph: {}", e))?;

    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &serializable)?;

    Ok(())
}

pub(crate) fn load_state(path: PathBuf) -> Result<Session, Error> {
    let file = std::fs::File::open(path.clone())?;
    let serialized: SerializableGraph<(bool, (f32, f32))> = serde_json::from_reader(file)?;

    let (graph, metadata) = serialized.from_serializable()?;

    let mut visible_nodes: std::collections::HashSet<DependencyId> =
        std::collections::HashSet::new();
    let mut node_positions: std::collections::HashMap<DependencyId, (f32, f32)> =
        std::collections::HashMap::new();

    let meta = metadata.ok_or(anyhow::anyhow!("Missing metadata in the file"))?;
    for (id, (visible, (x, y))) in meta.into_iter() {
        if visible {
            visible_nodes.insert(id);
        }
        node_positions.insert(id, (x, y));
    }

    Ok(Session::new(path, graph, node_positions, visible_nodes))
}
