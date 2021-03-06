use std::sync::Arc;
use entities::inspection::Repr;
use world::CollisionGroups;

/// A stand-alone object that has a position and a shape.
pub struct CollisionObject<N, P, V, M, T> {
    /// The position of the collision object.
    pub position: M,
    /// The shape of the collision object.
    pub shape: Arc<Box<Repr<N, P, V, M>>>,
    /// The collision groups of the collision object.
    pub collision_groups: CollisionGroups,
    /// The data associated to this object.
    pub data: T,
    #[doc(hidden)]
    pub timestamp: usize
}

impl<N, P, V, M, T> CollisionObject<N, P, V, M, T> {
    /// Creates a new collision object.
    pub fn new(position: M,
               shape:    Arc<Box<Repr<N, P, V, M>>>,
               groups:   CollisionGroups,
               data:     T)
               -> CollisionObject<N, P, V, M, T> {
        CollisionObject {
            position:         position,
            shape:            shape,
            collision_groups: groups,
            data:             data,
            timestamp:        0
        }
    }
}
