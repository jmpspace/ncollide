# Bounding volumes

Performing some tests on an approximation of the geometry of an object is often
useful to fasten several geometric queries.


For example testing two convex polyhedron for intersection is a very
time-consuming operation. Instead, we could test that their spherical
approximations (namely, their bounding spheres) intersect; and if the
approximations fails this intersection test, then there is no need to perform
the same query on the original polyhedra. This test-on-the-approximations-first
method is called _prunning_.


The approximations presented here are conservative with regard to the object
volume−that is−the approximated geometry's completely contained inside of the
approximating object. This is called a bounding volume. There are many possible
bounding bounding volumes.


The following figure shows a 2D polygon bounded by a Bounding Sphere, an Axis
Aligned Bounding Box (AABB), an Oriented Bounding Box (OBB), and a Convex Hull:

<center>
![bounding volumes](../img/bounding_volumes.svg)
</center>

Currently, **ncollide** only supports [Bounding
Spheres](../bounding_volumes/bounding_sphere.html) and
[AABB](../bounding_volumes/aabb.html). Also note that bounding volumes are very
different from regular geometries: their position in space is completely
contained by the bounding volume structure (no need to use it together with a
transformation matrix).


## Traits

Bounding volumes must implement the `bounding_volume::BoundingVolume` trait:


| Method            | Description |
|--                 | --          |
| `.intersects(bv)` | Checks `self` for intersection with `bv`.              |
| `.contains(bv)`   | Returns `true` if `bv` is completely inside of `self`. |
| `.merge(bv)`      | Merge `self` and `bv` in place. |
| `.merged(bv)`     | Returns a bounding volume, result of the merge of `self` with `bv`. |

Some bounding volume may also implement the
`bounding_volume::LooseBoundingVolume` trait. This gives the ability to enlarge
the volume by a given margin which is useful to optimize some [broad
phase](../collision_detection/broad_phase.html) algorithms:


| Method         | Description                               |
|--              | --                                        |
| `.loosen(m)`   | Enlarges `self` by `m` in place.          |
| `.loosened(m)` | Returns a copy of `self` enlarged by `m`. |