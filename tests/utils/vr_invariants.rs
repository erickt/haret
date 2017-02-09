// Copyright 2017 VMware, Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains assertion functions around VR protocol invariants. It's general enough to
//! be used from multiple tests.

use std::u64::MAX;
use haret::vr::VrCtx;

pub fn assert_single_primary_per_epoch_view(states: &Vec<(&'static str, VrCtx)>)
    -> Result<(), String>
{
    // List of epoch/views for all primaries
    let mut epoch_view = None;
    for &(state, ref ctx) in states {
        if state == "primary" {
            match epoch_view {
                None => epoch_view = Some((ctx.epoch, ctx.view)),
                Some((epoch, view)) => {
                    return safe_assert!(epoch != ctx.epoch || view != ctx.view)
                }
            }
        }
    }
    Ok(())
}

pub fn assert_minority_of_nodes_recovering(quorum: usize,
                                           states: &Vec<(&'static str, VrCtx)>)
    -> Result<(), String>
{
    let mut recovering_count = 0;
    for &(state, _) in states {
        if state == "recovery" {
            recovering_count += 1;
        }
    }
    safe_assert!(recovering_count < quorum)
}

pub fn assert_quorum_of_logs_equal_up_to_smallest_commit(quorum: usize,
                                                         states: &Vec<(&'static str, VrCtx)>)
    -> Result<(), String>
{
    let mut smallest_commit: u64 = MAX;
    for &(_, ref ctx) in states {
        if ctx.commit_num < smallest_commit {
            smallest_commit = ctx.commit_num;
        }
    }
    if smallest_commit == 0 { return Ok(()) }

    let mut slice = None;
    let mut count = 0;
    for &(_, ref ctx) in states {
        if ctx.commit_num >= smallest_commit {
            match slice {
                None => {
                    // We define the log prefix we will check in the next iteration
                    slice = Some(&ctx.log[0..smallest_commit as usize]);
                    count += 1;
                },
                Some(s) => {
                    // Are the log prefixes the same?
                    try!(safe_assert_eq!(s, &ctx.log[0..smallest_commit as usize]));
                    count += 1;
                }
            }
        }
    }
    safe_assert!(count >= quorum)
}
