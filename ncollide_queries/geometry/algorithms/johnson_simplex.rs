//!  Simplex using the Johnson subalgorithm to compute the projection of the origin on the simplex.

use std::mem;
use std::iter;
use std::cell::RefCell;
use std::sync::Arc;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use na::{Axpy, Dim, Bounded};
use na;
use geometry::algorithms::simplex::Simplex;
use math::{Scalar, Point, Vect};


thread_local!(static KEY_RECURSION_TEMPLATE: RefCell<Arc<Vec<RecursionTemplate>>> = RefCell::new(Arc::new(Vec::new())));

///  Simplex using the Johnson subalgorithm to compute the projection of the origin on the simplex.
#[derive(Clone)]
pub struct JohnsonSimplex<N, P, V> {
    recursion_template: Arc<Vec<RecursionTemplate>>,
    points:             Vec<P>,
    exchange_points:    Vec<P>,
    determinants:       Vec<N>,
    data:               PhantomData<V> // FIXME: remove this. V should be deduced by associated types instead.
}

/// Set of indices to explain to the JohnsonSimplex how to do its work.
/// Building this is very time consuming, and thus should be shared between all instances of the
/// Johnson simplex.
#[derive(PartialEq, Clone, RustcEncodable, RustcDecodable)]
pub struct RecursionTemplate {
    #[doc(hidden)]
    permutation_list: Vec<usize>,
    #[doc(hidden)]
    offsets:          Vec<usize>,
    #[doc(hidden)]
    sub_determinants: Vec<usize>,
    #[doc(hidden)]
    num_determinants: usize,
    #[doc(hidden)]
    num_leaves:       usize // useful only for printing…
}

impl RecursionTemplate {
    /// Creates a new set of Recursion simplex sharable between any Johnson simplex having a
    /// dimension inferior or equal to `dim`.
    pub fn new(dim: usize) -> Arc<Vec<RecursionTemplate>> {
        let mut template = Vec::with_capacity(dim + 1);

        for dim in 0usize .. dim + 1 {
            template.push(RecursionTemplate::make_permutation_list(dim))
        }

        Arc::new(template)
    }

    // pub fn to_raw_str(&self) -> ~str {
    //     let res = "permutation_list: " + self.permutation_list.to_str() + ", " +
    //               "offset: "           + self.offsets.to_str() + ", " +
    //               "sub_determinants: " + self.sub_determinants.to_str();

    //     res
    // }

    // This is the tricky part of the algorithm. This generates all datas needed
    // to run the johson subalgorithm fastly. This should _not_ be run every time
    // the algorithm is executed. Instead, it should be pre-computed, or computed
    // only once for all. The resulting GC-managed list is intented to be shared
    // between all other simplicis with the same dimension.
    fn make_permutation_list(dim: usize) -> RecursionTemplate {
        // The number of points on the biggest subsimplex
        let max_num_points      = dim + 1;

        let mut pts              = Vec::new(); // the result
        let mut offsets          = Vec::new();
        let mut sub_determinants = Vec::new();

        // the beginning of the last subsimplices list
        let mut last_dim_begin  = 0;

        // the end of the last subsimplices list
        let mut last_dim_end    = dim + 1 + 1;

        // the number of points of the last subsimplices
        let mut last_num_points = dim + 1;

        let mut map             = BTreeMap::<Vec<usize>, usize>::new();

        let mut determinant_index  = 0;

        for i in 0 .. max_num_points {
            pts.push(i)
        }

        // initially push the whole simplex (will be removed at the end)
        pts.push(0);

        offsets.push(max_num_points + 1);

        // ... then remove one point each time
        for i in 0usize .. dim + 1 {
            // for each sub-simplex ...
            let mut curr      = last_dim_begin;
            let mut num_added = 0;

            while curr != last_dim_end {
                // ... iterate on it ...
                for j in 0usize .. last_num_points {
                    // ... and build all the sublist with one last point
                    let mut sublist = Vec::new();

                    // then extract the sub-simplex
                    for k in 0usize .. last_num_points {
                        // we remove the j'th point
                        if pts[curr + j] != pts[curr + k] {
                            sublist.push(pts[curr + k]);
                        }
                    }

                    // keep a trace of the removed point
                    sublist.push(pts[curr + j]);

                    match map.get(&sublist) {
                        Some(&v) => sub_determinants.push(v),
                        None     => {
                            for &e in sublist.iter() {
                                pts.push(e);
                                num_added = num_added + 1;
                            }
                            sub_determinants.push(determinant_index);
                            let _ = map.insert(sublist, determinant_index);
                            determinant_index = determinant_index + 1;
                        }
                    }
                }

                let mut parent = Vec::new();
                for k in 0usize .. last_num_points + 1 {
                    parent.push(pts[curr + k])
                }


                match map.get(&parent) {
                    Some(&p) => sub_determinants.push(p),
                    None => {
                        sub_determinants.push(determinant_index);
                        // There is no need to keep a place for the full simplex determinant.
                        // So we dont increase the determinant buffer index for the first
                        // iteration.
                        determinant_index = determinant_index + if i == 0 { 0 } else { 1 };
                    }
                }

                curr = curr + last_num_points + 1;
            }

            // initialize the next iteration with one less point
            last_dim_begin = last_dim_end ;
            last_dim_end = last_dim_end + num_added;
            offsets.push(last_dim_end);
            last_num_points = last_num_points - 1;
        }

        // determinant indices for leaves
        for i in 0usize .. max_num_points {
            sub_determinants.push(*map.get(&vec!(max_num_points - 1 - i)).unwrap())
        }

        // end to begin offsets
        offsets.insert(0, 0usize);
        offsets.reverse();
        let _ = offsets.pop();

        let rev_offsets: Vec<usize> = offsets.iter().map(|&e| pts.len() - e).collect();
        let num_leaves = rev_offsets[0];

        // reverse points and detereminants
        pts.reverse();
        sub_determinants.reverse();

        // remove the full simplex
        let num_pts = pts.len();
        pts.truncate(num_pts - max_num_points - 1);
        sub_determinants.truncate(num_pts - max_num_points - 1);

        RecursionTemplate {
            offsets:          rev_offsets,
            permutation_list: pts,
            num_determinants: sub_determinants[0] + 1,
            sub_determinants: sub_determinants,
            num_leaves:       num_leaves
        }
    }
}

impl<N, P, V> JohnsonSimplex<N, P, V>
    where N: Scalar,
          P: Dim {
    /// Creates a new, empty, Johnson simplex.
    pub fn new(recursion: Arc<Vec<RecursionTemplate>>) -> JohnsonSimplex<N, P, V> {
        let _dim = na::dim::<P>();

        JohnsonSimplex {
            points:             Vec::with_capacity(_dim + 1),
            exchange_points:    Vec::with_capacity(_dim + 1),
            determinants:       iter::repeat(na::zero()).take(recursion[_dim].num_determinants).collect(),
            recursion_template: recursion,
            data:               PhantomData
        }
    }

    /// Creates a new, empty Johnson simplex. The recursion template uses the thread-local one.
    pub fn new_w_tls() -> JohnsonSimplex<N, P, V> {
        KEY_RECURSION_TEMPLATE.with(|rec| {
            if rec.borrow().len() <= na::dim::<P>() {
                *rec.borrow_mut() = RecursionTemplate::new(na::dim::<P>());
            }
            JohnsonSimplex::new(rec.borrow().clone())
        })
    }
}

impl<N, P, V> JohnsonSimplex<N, P, V>
    where N: Scalar,
          P: Point<N, V>,
          V: Vect<N> {
    fn do_project_origin(&mut self, reduce: bool) -> P {
        if self.points.is_empty() {
            panic!("Cannot project the origin on an empty simplex.")
        }

        if self.points.len() == 1 {
            return self.points[0].clone();
        }

        let max_num_pts      = self.points.len();
        let recursion        = &self.recursion_template[max_num_pts - 1];
        let mut curr_num_pts = 1usize;
        let mut curr         = max_num_pts;

        let ndets = self.determinants.len();
        for c in self.determinants.slice_mut(recursion.num_determinants - max_num_pts, ndets).iter_mut() {
            *c = na::one();
        }

        // NOTE: Please read that before thinking all those `unsafe_whatever` should be bannished.
        // The whole point of having this `recursion_template` stuff is to speed up the
        // computations having exact precomputed indices.
        // Using safe accesses to vectors kind of makes this useless sinces each array access will
        // be much slower.
        // That is why we use unsafe indexing here. Nothing personal, just a huge need of
        // performances :p
        // There might be a whay to to this nicely with iterators. But indexing is verry intricate
        // here…

        /*
         * first loop: compute all the determinants
         */
        for &end in recursion.offsets.slice_from(2).iter() { // FIXME: try to transform this using a `window_iter()`
            // for each sub-simplex ...
            while curr != end { // FIXME: replace this `while` by a `for` when a range with custom increment exist
                unsafe {
                    let mut determinant: N = na::zero();
                    let kpt = (*self.points.as_slice().get_unchecked(*recursion.permutation_list.as_slice().get_unchecked(curr + 1usize))).clone();
                    let jpt = (*self.points.as_slice().get_unchecked(*recursion.permutation_list.as_slice().get_unchecked(curr))).clone();

                    // ... with curr_num_pts points ...
                    for i in curr + 1 .. curr + 1 + curr_num_pts {
                        // ... compute its determinant.
                        let i_pid = *recursion.permutation_list.as_slice().get_unchecked(i);
                        let sub_determinant = (*self.determinants.as_slice().get_unchecked(
                                                *recursion.sub_determinants.as_slice().get_unchecked(i))).clone();
                        let delta = sub_determinant * na::dot(&(kpt - jpt), self.points.as_slice().get_unchecked(i_pid).as_vec());

                        determinant = determinant + delta;
                    }

                    *self.determinants.as_mut_ptr().offset(*recursion.sub_determinants.as_slice().get_unchecked(curr) as isize) = determinant;

                    curr = curr + curr_num_pts + 1; // points + removed point + determinant id
                }
            }

            curr_num_pts = curr_num_pts + 1;
        }

        /*
         * second loop: find the subsimplex containing the projection
         */
        let mut offsets_iter = recursion.offsets.as_slice().iter().rev();
        let     _            = offsets_iter.next(); // skip the first offset
        for &end in offsets_iter {
            // for each sub-simplex ...
            while curr != end {
                let mut foundit = true;

                // ... with curr_num_pts points permutations ...
                for i in 0usize .. curr_num_pts {
                    unsafe {
                        // ... see if its determinant is positive
                        let det_id = curr - (i + 1) * curr_num_pts;
                        let det    = (*self.determinants.as_slice().get_unchecked(*recursion.sub_determinants.as_slice().get_unchecked(det_id))).clone();

                        if det > na::zero() {
                            // invalidate the children determinant
                            if curr_num_pts > 1 {
                                let subdetid = *recursion.sub_determinants.as_slice().get_unchecked(det_id + 1);

                                if *self.determinants.as_slice().get_unchecked(subdetid) > na::zero() {
                                    *self.determinants.as_mut_ptr().offset(subdetid as isize) = Bounded::max_value()
                                }
                            }

                            // dont concider this sub-simplex if it has been invalidated by its
                            // parent(s)
                            if det == Bounded::max_value() {
                                foundit = false
                            }
                        }
                        else {
                            // we found a negative determinant: no projection possible here
                            foundit = false
                        }
                    }
                }

                if foundit {
                    // we found a projection!
                    // re-run the same iteration but, this time, compute the projection
                    let mut total_det: N = na::zero();
                    let mut proj: P      = na::orig();

                    unsafe {
                        for i in 0usize .. curr_num_pts { // FIXME: change this when decreasing loops are implemented
                            // ... see if its determinant is positive
                            let id    = curr - (i + 1) * curr_num_pts;
                            let det   = (*self.determinants
                                              .as_slice()
                                              .get_unchecked(*recursion.sub_determinants.as_slice().get_unchecked(id))).clone();

                            total_det = total_det + det;
                            proj.axpy(&det, self.points.as_slice().get_unchecked(*recursion.permutation_list.as_slice().get_unchecked(id)));
                        }

                        if reduce {
                            // we need to reduce the simplex
                            for i in 0usize .. curr_num_pts {
                                let id = curr - (i + 1) * curr_num_pts;
                                self.exchange_points.push(
                                    (*self.points.as_slice().get_unchecked(
                                        *recursion.permutation_list.as_slice().get_unchecked(id))).clone());
                            }

                            mem::swap(&mut self.exchange_points, &mut self.points);
                            self.exchange_points.clear();
                        }
                    }

                    return proj / total_det;
                }

                curr = curr - curr_num_pts * curr_num_pts;
            }

            curr_num_pts = curr_num_pts - 1;
        }

        na::orig()
    }
}

impl<N, P, V> Simplex<N, P> for JohnsonSimplex<N, P, V>
    where N: Scalar,
          P: Point<N, V>,
          V: Vect<N> {
    #[inline]
    fn reset(&mut self, pt: P) {
        self.points.clear();
        self.points.push(pt);
    }

    #[inline]
    fn dimension(&self) -> usize {
        self.points.len() - 1
    }

    #[inline]
    fn max_sq_len(&self) -> N {
        let mut max_sq_len = na::zero();

        for p in self.points.iter() {
            let norm = na::sqnorm(p.as_vec());

            if norm > max_sq_len {
                max_sq_len = norm
            }
        }

        max_sq_len
    }

    #[inline]
    fn contains_point(&self, pt: &P) -> bool {
        self.points.iter().any(|v| pt == v)
    }

    #[inline]
    fn add_point(&mut self, pt: P) {
        self.points.push(pt);
        assert!(self.points.len() <= na::dim::<P>() + 1);
    }

    #[inline]
    fn project_origin_and_reduce(&mut self) -> P {
        self.do_project_origin(true)
    }

    #[inline]
    fn project_origin(&mut self) -> P {
        self.do_project_origin(false)
    }

    #[inline(always)]
    fn modify_pnts(&mut self, f: &Fn(&mut P)) {
        for pt in self.points.iter_mut() {
            f(pt)
        }
    }
}

// impl ToStr for RecursionTemplate {
//     fn to_str(&self) -> ~str {
//         let mut res  = ~"RecursionTemplate { ";
//         let mut curr = self.num_leaves;
//         let mut dim  = 1;
//
//         res = res + "num_determinants: " + self.num_determinants.to_str();
//
//         let mut recursion_offsets_skip_1 = self.offsets.iter();
//         let     _                        = recursion_offsets_skip_1.next(); // Skip the two first entries
//
//         for &off in recursion_offsets_skip_1 {
//             while curr != off {
//                 res = res + "\n(@" + self.sub_determinants[curr].to_str() + " -> ";
//
//                 for i in 0u .. dim {
//                     res = res + self.permutation_list[i + curr].to_str();
//                     if i != dim - 1 {
//                         res = res + " ";
//                     }
//                 }
//
//                 res = res + " - ";
//
//                 for i in 1u .. dim {
//                     res = res + self.sub_determinants[i + curr].to_str();
//                     if i != dim - 1 {
//                         res = res + " ";
//                     }
//                 }
//
//                 res  = res + ")";
//                 curr = curr + dim;
//             }
//
//             dim = dim + 1;
//         }
//
//         res = res + " }\n";
//
//         res = res + "offsets: " + self.offsets.to_str();
//
//         res
//     }
// }
