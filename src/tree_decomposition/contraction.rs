use clap::ValueEnum;
use rustc_hash::{FxHashMap, FxHashSet};
use rand::seq::SliceRandom;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};


use crate::restricted::{Restriction, RestrictionOp};
use crate::problem::Problem;

