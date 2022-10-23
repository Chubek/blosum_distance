#![allow(unstable_name_collisions, unused_mut)]

use hashbrown::HashMap;
use itertools::Itertools;
use pyo3::prelude::*;
use rayon::prelude::*;

/*
Notes to Igster and Kevin:

I tried my best to use parallelism within the outer iterator. But within the inner
iterator I still can't find a pattern to make it parallel. That's because par_iter()
does uses FnOne() that does not accept mutable variables.

 I removed other functions in this branch for clarity. Feel free to rename the functions 
 and merge it with master.

 Also I changed &str to String because PyO3 can't accept references to the stack easily.
 However if you feel like &str is faster please feel free to convert them back. Mind you that
 par_iter() may cause lifetime troubles again for the use of FnOnce() in it. I'm not sure why that happens.

 What I did besides that was heavy use of iterators. That's basically it.

 Please test it and let me know if it's faster or slower. I would do it myself but I'm very tired right now.

 Since Igster is sick I will test it when I wake up later today/tonight.

 Thanks.


*/


#[pyfunction]
fn cluster_distance_filter(lines: Vec<String>) -> Vec<Vec<String>> {
    let mut clusters = HashMap::new();

    lines
        .iter()
        .cloned()
        .batching(|mut it| match it.next() {
            None => None,
            Some(x) => match it.next() {
                None => None,
                Some(y) => Some((x, y)),
            },
        })
        .for_each(|(name, seq)| {
            let key = (seq.len(), seq[0..10].to_string());
            let tuplet = (name, seq);

            clusters.entry(key).or_insert(Vec::new()).push(tuplet);
        });

    clusters
        .par_iter()
        .map(|(_, cluster)| {
            let mut this_lead = vec![];

            cluster
                .iter()
                .cloned()
                .batching(|mut it| match it.next() {
                    None => None,
                    Some(x) => Some(it.clone().intersperse(x)),
                })
                .for_each(|it| {
                    it.batching(|mut iti| match iti.next() {
                        None => None,
                        Some(x) => match iti.next() {
                            None => None,
                            Some(y) => Some((x, y)),
                        },
                    })
                    .for_each(|((header, candidate), (_, lead))| {
                        match seqs_within_distance(candidate.clone(), lead, 1) {
                            true => {
                                this_lead.extend(vec![
                                    header,
                                    candidate,
                                ]);
                            }
                            false => (),
                        }
                    });
                });
            this_lead
        })
        .collect()
}

#[pyfunction]
fn seqs_within_distance(first: String, second: String, max_distance: u32) -> bool {
    let (array_one, array_two) = (first.as_bytes(), second.as_bytes());
    if array_one.len() != array_two.len() {
        return false;
    }
    let mut distance: u32 = 0;
    for i in 0..array_one.len() {
        if array_one[i] != array_two[i] {
            distance += 1;
            if distance > max_distance {
                return false;
            }
        }
    }
    true
}

// A Python module implemented in Rust.
#[pymodule]
fn blosum_distance(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cluster_distance_filter, m)?)?;
    m.add_function(wrap_pyfunction!(seqs_within_distance, m)?)?;
    Ok(())
}
