use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProcessState {
    New,
    Ready,
    Running,
    Suspended,
    Terminated,
}

#[derive(Debug, Ord, Eq, PartialEq, PartialOrd, Clone, Copy)]
struct Process {
    start_time: u64,
    remaining_time: u64,
    serviced_time: u64,
    waiting_time: u64,
    priority: u64,
    id: usize,
    state: ProcessState,
}

impl Process {
    fn new(id: usize, start_time: u64, execution_time: u64, priority: u64) -> Self {
        Self {
            id,
            start_time,
            remaining_time: execution_time,
            priority,
            waiting_time: 0,
            serviced_time: 0,
            state: ProcessState::New,
        }
    }
}

type Strategy = fn(&mut Processor);

struct Processor {
    current: Option<usize>,
    processes: Vec<Process>,
    strategy: Strategy,
    system_time: u64,
}

impl Processor {
    fn new(processes: Vec<Process>, strategy: Strategy) -> Self {
        Self {
            current: None,
            processes,
            strategy,
            system_time: 0,
        }
    }

    fn first_come_first_serve(processes: Vec<Process>) -> Self {
        Self::new(processes, first_come_first_serve)
    }

    fn shortest_job_first(processes: Vec<Process>) -> Self {
        Self::new(processes, shortest_job_first)
    }

    fn lowest_priority_first(processes: Vec<Process>) -> Self {
        todo!("lowest_priority_first")
    }

    fn lowest_priority_first_preemptive(processes: Vec<Process>) -> Self {
        todo!("lower_priority_first_preemptive")
    }

    fn shortest_remaining_time(processes: Vec<Process>) -> Self {
        todo!("shortest_remaining_time")
    }

    fn round_robin(processes: Vec<Process>) -> Self {
        todo!("round_robin")
    }

    fn multi_level_feedback_queue(processes: Vec<Process>) -> Self {
        todo!("multi_level_feedback_queue")
    }

    fn tick(&mut self) -> u64 {

        (&self.strategy)(self);
        self.system_time += 1;

        self.system_time
    }

    fn run(&mut self) {
        let mut log: Vec<SchedulingEvent> = Vec::new();

        for _ in 0..30 {
            let id_prev = self.current_process_id();
            let system_time = self.tick();
            let id_curr = self.current_process_id();

            let prev_process = {
                match id_prev {
                    Some(id) => Some(self.get_process(id)),
                    None => None
                }
            };

            let curr_process = {
                match id_curr {
                    Some(id) => Some(self.get_process(id)),
                    None => None
                }
            };

            let event = SchedulingEvent::new(system_time, prev_process, curr_process);
            log.push(event);
        }

        for event in log {
            println!("{}", event);
        }
    }

    fn get_process(&self, id: usize) -> &Process {
            self.processes.iter().find(|p| p.id == id).unwrap()
    }

    fn current_process_mut(&mut self) -> Option<&mut Process> {
        if let Some(id) = self.current {
            return Some(self.processes.iter_mut().find(|p| p.id == id).unwrap());
        }
        None
    }

    fn current_process_ref(&self) -> Option<&Process> {
        if let Some(id) = self.current {
            return Some(self.get_process(id));
        }
        None
    }

    fn current_process_id(&self) -> Option<usize> {
        if let Some(process) = self.current_process_ref() {
            return Some(process.id);
        }
        None
    }

    fn set_current_process_state(&mut self, state: ProcessState) {
        if let Some(process_mut) = self.current_process_mut() {
            process_mut.state = state;
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct SchedulingEvent {
    system_time: u64,
    start_of_tick: Option<(usize, ProcessState)>,
    end_of_tick: Option<(usize, ProcessState)>
}

impl SchedulingEvent {
    fn new(system_time: u64, start_of_tick: Option<&Process>, end_of_tick: Option<&Process>) -> Self {
        let start = {
            match start_of_tick {
                Some(process) => Some((process.id, process.state)),
                None => None,
            }
        };
        let end = {
            match end_of_tick {
                Some(process) => Some((process.id, process.state)),
                None => None,
            }
        };
        Self {
            system_time,
            start_of_tick: start,
            end_of_tick: end,
        }
    }
}

impl fmt::Display for SchedulingEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:05}: ", self.system_time)?;
        match self.start_of_tick {
            Some((id, state)) => write!(f, "{state:?}({id})")?,
            None => write!(f, "None")?,
        };
        write!(f, " -> ")?;
        match self.end_of_tick{
            Some((id, state)) => write!(f, "{state:?}({id})")?,
            None => write!(f, "None")?,
        };
        Ok(())
    }
}

fn first_come_first_serve(processor: &mut Processor) {
    if let Some(current) = processor.current_process_mut() {
        current.remaining_time -= 1;
        current.serviced_time += 1;
        if current.remaining_time > 0 {
            return;
        }

        processor.set_current_process_state(ProcessState::Terminated);
        processor.current = None;
    }

    // At this point, the processor has no currently executed process
    for process in &processor.processes {
        if process.remaining_time > 0 {
            processor.current = Some(process.id);
            break;
        }
    }

    processor.set_current_process_state(ProcessState::Running);
}

fn shortest_job_first(processor: &mut Processor) {
    if let Some(current) = processor.current_process_mut() {
        current.remaining_time -= 1;
        current.serviced_time += 1;
        if current.remaining_time > 0 {
            return;
        }

        processor.set_current_process_state(ProcessState::Terminated);
        processor.current = None;
    }

    // At this point, the processor has no currently executed process
    let (mut id, mut len) = (None, None);
    for process in processor.processes.iter().filter(|p| p.remaining_time != 0) {
        if process.remaining_time < len.unwrap_or(u64::MAX) {
            (id, len) = (Some(process.id), Some(process.remaining_time));
        }
    }

    processor.current = id;
    processor.set_current_process_state(ProcessState::Running);
}

fn main() {
    let mut processes = vec![
        Process::new(0, 1, 2, 2),
        Process::new(1, 3, 3, 3),
        Process::new(2, 3, 1, 5),
        Process::new(3, 0, 4, 1),
        Process::new(4, 2, 2, 1),
    ];

    processes.sort();

    Processor::first_come_first_serve(processes.clone()).run();
    Processor::shortest_job_first(processes.clone()).run();
}
