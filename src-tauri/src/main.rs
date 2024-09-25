// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use serde::Serialize;
use sorted_vec::ReverseSortedVec;
use tauri::State;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum SchedulingAlgorithms {
    FCFS,
    SJF,
    RR,
    PRIORITY,
}

#[derive(Serialize)]
pub enum Inputs {
    None,
    Priority,
    Quantum,
}

pub struct AppState {
    pub algorithm: Mutex<Option<SchedulingAlgorithms>>,
}

#[derive(Serialize)]
pub struct ProcessSegment {
    pub process_id: Option<String>,
    pub end_time: u32,
}

pub type GanttChart = Vec<ProcessSegment>;

pub type SegmentInput = (String, u32, u32);
pub type PrioritySegmentInput = (String, u32, u32, u32);

#[derive(Serialize)]
pub struct Times {
    pub completion: u32,
    pub waiting: u32,
    pub turnaround: u32,
}

impl Default for Times {
    fn default() -> Self {
        Self {
            completion: 0,
            waiting: 0,
            turnaround: 0,
        }
    }
}

//Name, CT, AT, WT
pub type OutputTable = HashMap<String, Times>;

#[derive(PartialEq, Eq, PartialOrd)]
pub struct ATBasedSegment(SegmentInput);

impl Ord for ATBasedSegment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.2.cmp(&other.2)
    }
}

impl std::ops::Deref for ATBasedSegment {
    type Target = SegmentInput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(PartialEq, Eq, PartialOrd, Debug)]
pub struct BTBasedSegment(SegmentInput);

impl Ord for BTBasedSegment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

impl std::ops::Deref for BTBasedSegment {
    type Target = SegmentInput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(PartialEq, Eq, PartialOrd, Debug)]
pub struct PriorityBasedSegment(PrioritySegmentInput);

impl Default for AppState {
    fn default() -> Self {
        AppState {
            algorithm: Mutex::new(None),
        }
    }
}

impl Ord for PriorityBasedSegment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.3.cmp(&other.3)
    }
}

impl std::ops::Deref for PriorityBasedSegment {
    type Target = PrioritySegmentInput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[tauri::command]
fn change_state(algo: String, raw_state: State<AppState>) -> Result<Inputs, ()> {
    let mut state = raw_state.algorithm.lock().unwrap();
    *state = match algo.as_str() {
        "fcfs" => Some(SchedulingAlgorithms::FCFS),
        "sjf" => Some(SchedulingAlgorithms::SJF),
        "rr" => Some(SchedulingAlgorithms::RR),
        "priority" => Some(SchedulingAlgorithms::PRIORITY),
        _ => None,
    };
    println!("State is now {:?}", state);
    if state.is_none() {
        return Err(());
    }
    match state.clone().unwrap() {
        SchedulingAlgorithms::FCFS => Ok(Inputs::None),
        SchedulingAlgorithms::PRIORITY => Ok(Inputs::Priority),
        SchedulingAlgorithms::SJF => Ok(Inputs::None),
        SchedulingAlgorithms::RR => Ok(Inputs::Quantum),
    }
}

#[tauri::command]
fn get_inputs(raw_state: State<AppState>) -> Inputs {
    let state = raw_state.algorithm.lock().unwrap();
    let state = state
        .clone()
        .expect("Cannot get input without changing state");
    match state {
        SchedulingAlgorithms::FCFS => Inputs::None,
        SchedulingAlgorithms::PRIORITY => Inputs::Priority,
        SchedulingAlgorithms::SJF => Inputs::None,
        SchedulingAlgorithms::RR => Inputs::Quantum,
    }
}

#[tauri::command]
fn get_state(raw_state: State<AppState>) -> SchedulingAlgorithms {
    let state = raw_state.algorithm.lock().unwrap();
    let state = state
        .clone()
        .expect("Cannot get state without setting it first");
    return state;
}

#[tauri::command]
fn get_gantt_fcfs(
    raw_state: State<AppState>,
    inputs: Vec<SegmentInput>,
) -> Result<(GanttChart, OutputTable), ()> {
    let state = raw_state.algorithm.lock().unwrap();
    let state = state.clone().expect("");

    if state != SchedulingAlgorithms::FCFS {
        panic!("State dettached!")
    }

    let mut output = inputs.clone();
    let mut values = inputs
        .clone()
        .into_iter()
        .map(|segment| (segment.0, Times::default()))
        .collect::<OutputTable>();
    println!("Before sort: {:?}", output);
    output.sort_by(|a, b| a.2.cmp(&b.2));
    println!("After sort: {:?}", output);

    let mut acc = 0;

    let mut proc_segs = vec![];

    for segment in output.into_iter() {
        if acc < segment.2 {
            proc_segs.push(ProcessSegment {
                process_id: None,
                end_time: segment.2,
            });
            acc = segment.2;
        }
        values
            .entry(segment.0.clone())
            .and_modify(|times| times.waiting = acc - segment.1);
        acc += segment.1;
        values
            .entry(segment.0.clone())
            .and_modify(|times| {
                times.completion = acc;
                times.turnaround = acc - segment.2;
            });
        proc_segs.push(ProcessSegment {
            process_id: Some(segment.0),
            end_time: acc,
        });
    }

    Ok((proc_segs, values))
}

#[tauri::command]
fn get_gantt_sjf(
    raw_state: State<AppState>,
    inputs: Vec<SegmentInput>,
) -> Result<(GanttChart, OutputTable), ()> {
    let state = raw_state.algorithm.lock().unwrap();
    let state = state.clone().expect("");

    if state != SchedulingAlgorithms::SJF {
        panic!("State dettached!")
    }

    let mut output = inputs.clone();
    let mut values: OutputTable = inputs
        .clone()
        .into_iter()
        .map(|segments| (segments.0, Times::default()))
        .collect();
    output.sort_by(|a, b| a.2.cmp(&b.2));

    let mut output = output.into_iter().peekable();

    let mut acc = 0;
    let mut proc_segs = vec![];
    let mut not_handled: ReverseSortedVec<BTBasedSegment> = ReverseSortedVec::new();

    loop {
        if output.peek().is_none() && not_handled.is_empty() {
            break;
        }
        while let Some(next_proc) = output.next_if(|segment| acc >= segment.2) {
            let bt_segment = BTBasedSegment(next_proc);
            not_handled.insert(std::cmp::Reverse(bt_segment));
        }
        println!(
            "{:?}, {:?}",
            output.clone().collect::<Vec<_>>(),
            not_handled
        );
        match not_handled.pop() {
            None => {
                let new_time = output.peek().clone().unwrap().2;
                proc_segs.push(ProcessSegment {
                    process_id: None,
                    end_time: new_time,
                });
                acc = new_time;
                continue;
            }
            Some(std::cmp::Reverse(BTBasedSegment(next_segment))) => {
                values
                    .entry(next_segment.0.clone())
                    .and_modify(|times| times.waiting = acc - next_segment.2);
                acc += next_segment.1;
                values.entry(next_segment.0.clone()).and_modify(|times| {
                    times.completion = acc;
                    times.turnaround = acc - next_segment.2;
                });
                proc_segs.push(ProcessSegment {
                    process_id: Some(next_segment.0),
                    end_time: acc,
                });
            }
        }
    }

    Ok((proc_segs, values))
}

#[tauri::command]
fn get_gantt_rr(
    raw_state: State<AppState>,
    quantum: u32,
    inputs: Vec<SegmentInput>,
) -> Result<(GanttChart, OutputTable), ()> {
    let state = raw_state.algorithm.lock().unwrap();
    let state = state.clone().expect("");

    if state != SchedulingAlgorithms::RR {
        panic!("State dettached!")
    }

    let mut output = inputs.clone();
    output.sort_by(|a, b| a.2.cmp(&b.2));

    let mut acc = 0;
    let mut proc_segs = vec![];
    let mut proc_tracking: HashMap<String, u32> = output
        .clone()
        .into_iter()
        .map(|segment| (segment.0, segment.1))
        .collect::<HashMap<_, _>>();
    let mut output = output.into_iter().peekable();
    let mut values: OutputTable = inputs
        .clone()
        .into_iter()
        .map(|segment| (segment.0, Times::default()))
        .collect();
    let mut ready_queue: VecDeque<SegmentInput> = VecDeque::new();

    if let Some(next_proc) = output.next() {
        if next_proc.2 > 0 {
            proc_segs.push(ProcessSegment {
                process_id: None,
                end_time: next_proc.2,
            });
        }
        ready_queue.push_back(next_proc);
    }

    while output.peek().is_some() || !ready_queue.is_empty() {
        match ready_queue.pop_front() {
            None => {
                let next_time = output.peek().clone().unwrap().2;
                proc_segs.push(ProcessSegment {
                    process_id: None,
                    end_time: next_time,
                });
                acc = next_time;

                while let Some(next_proc) = output.next_if(|segment| acc >= segment.2) {
                    ready_queue.push_back(next_proc);
                }
                continue;
            }
            Some(segment) => {
                let rem_time = proc_tracking.get_mut(&segment.0).unwrap();
                let inc_amt = quantum.min(*rem_time);
                acc += inc_amt;
                proc_segs.push(ProcessSegment {
                    process_id: Some(segment.0.clone()),
                    end_time: acc,
                });
                *rem_time -= inc_amt;
                if *rem_time == 0 {
                    values.entry(segment.0.clone()).and_modify(|times| {
                        times.completion = acc;
                        times.waiting = acc - inc_amt - (segment.1.div_ceil(quantum) - 1) * quantum;
                        times.turnaround = acc - segment.2;
                    });
                }

                while let Some(next_proc) = output.next_if(|segment| acc >= segment.2) {
                    println!("Test");
                    ready_queue.push_back(next_proc);
                }

                if *rem_time > 0 {
                    ready_queue.push_back(segment);
                }
            }
        };
    }

    Ok((proc_segs, values))
}

#[tauri::command]
fn get_gantt_priority(
    raw_state: State<AppState>,
    inputs: Vec<PrioritySegmentInput>,
) -> Result<(GanttChart, OutputTable), ()> {
    let state = raw_state.algorithm.lock().unwrap();
    let state = state.clone().expect("");

    if state != SchedulingAlgorithms::PRIORITY {
        panic!("State dettached!")
    }

    let mut output = inputs.clone();
    let mut values: OutputTable = inputs
        .clone()
        .into_iter()
        .map(|segment| (segment.0, Times::default()))
        .collect();
    output.sort_by(|a, b| a.2.cmp(&b.2));

    let mut output = output.into_iter().peekable();

    let mut acc = 0;
    let mut proc_segs = vec![];
    let mut not_handled: ReverseSortedVec<PriorityBasedSegment> = ReverseSortedVec::new();

    loop {
        if output.peek().is_none() && not_handled.is_empty() {
            break;
        }
        while let Some(next_proc) = output.next_if(|segment| acc >= segment.2) {
            let bt_segment = PriorityBasedSegment(next_proc);
            not_handled.insert(std::cmp::Reverse(bt_segment));
        }
        println!(
            "{:?}, {:?}",
            output.clone().collect::<Vec<_>>(),
            not_handled
        );
        match not_handled.pop() {
            None => {
                let new_time = output.peek().clone().unwrap().2;
                proc_segs.push(ProcessSegment {
                    process_id: None,
                    end_time: new_time,
                });
                acc = new_time;
                continue;
            }
            Some(std::cmp::Reverse(PriorityBasedSegment(next_segment))) => {
                values
                    .entry(next_segment.0.clone())
                    .and_modify(|times| times.waiting = acc - next_segment.2);
                acc += next_segment.1;
                values
                    .entry(next_segment.0.clone())
                    .and_modify(|times| {
                        times.completion = acc;
                        times.turnaround = acc - next_segment.2;
                    });
                proc_segs.push(ProcessSegment {
                    process_id: Some(next_segment.0),
                    end_time: acc,
                });
            }
        }
    }

    Ok((proc_segs, values))
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            change_state,
            get_state,
            get_inputs,
            get_gantt_fcfs,
            get_gantt_sjf,
            get_gantt_rr,
            get_gantt_priority,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
