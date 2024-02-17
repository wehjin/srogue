#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use rand::{random, Rng, thread_rng};
use rand::distributions::uniform::SampleUniform;
use rand::prelude::SliceRandom;

pub fn get_rand<T: SampleUniform + PartialOrd>(x: T, y: T) -> T {
	thread_rng().gen_range(x..=y)
}

pub fn get_rand_indices(max_index: usize) -> Vec<usize> {
	let mut indices: Vec<usize> = (0..max_index).collect();
	indices.shuffle(&mut thread_rng());
	indices
}

pub fn rand_percent(percentage: usize) -> bool { get_rand(1, 100) <= percentage }

pub fn coin_toss() -> bool { random() }
